#!/usr/bin/env python3
"""
Validates commands to determine if they are safe/non-destructive.
Supports deep evaluation for Zsh, ADB, and Python commands.

Returns a JSON decision according to the Antigravity PreToolUse hook contract:
- "allow": automatically execute non-destructive commands.
- "ask": require user confirmation for potentially destructive commands.
"""

import ast
import json
import re
import shlex
import sys

# Standalone system binaries or commands that are destructive
DESTRUCTIVE_SYSTEM_COMMANDS = {
    "rm", "unlink", "rmdir", "shred", "wipe", "srm",
    "mkfs", "mke2fs", "mkfs.ext4", "fdisk", "parted", "gdisk", "sfdisk", "dd",
    "chmod -R", "chown -R", "chgrp -R",
    "reboot", "shutdown", "poweroff", "halt", "init",
    "apt-get remove", "apt remove", "apt-get purge", "apt purge",
    "pacman -R", "dnf remove", "yum remove", "pip uninstall", "uv pip uninstall",
}

# Regex patterns for destructive git operations
DESTRUCTIVE_GIT_PATTERNS = [
    r"\bgit\s+reset\b",
    r"\bgit\s+clean\b",
    r"\bgit\s+restore\b",
    r"\bgit\s+checkout\s+(--\s+|\.)",
    r"\bgit\s+branch\s+(-D|--delete\s+--force)",
    r"\bgit\s+push\s+.*(--force|-f)\b",
    r"\bgit\s+rebase\b",
]

# Database destructive operations
DESTRUCTIVE_DB_PATTERNS = [
    r"\bDROP\s+TABLE\b",
    r"\bDROP\s+DATABASE\b",
    r"\bTRUNCATE\b",
    r"\bDELETE\s+FROM\b",
]

# Destructive standalone ADB subcommands
DESTRUCTIVE_ADB_SUBCOMMANDS = {
    "uninstall",
    "reboot",
    "remount",
    "root",
    "unroot",
    "sideload",
    "restore",
    "disable-verity",
    "enable-verity",
}

# Destructive commands inside `adb shell`
DESTRUCTIVE_ADB_SHELL_COMMANDS = {
    "rm", "unlink", "rmdir", "shred", "wipe", "dd",
    "mkfs", "fdisk", "parted", "format",
    "reboot", "recovery", "bootloader",
    "setprop", "su",
}

# Python functions/methods that destroy data or execute unverified shell commands
DESTRUCTIVE_PYTHON_CALLS = {
    "remove", "unlink", "rmdir", "removedirs",
    "rmtree", "truncate",
}


def split_shell_pipeline(cmd_line: str) -> list[str]:
    """Splits shell commands separated by ;, &&, ||, &, or newlines while respecting quotes."""
    segments = []
    current = []
    in_single = False
    in_double = False
    escaped = False

    i = 0
    n = len(cmd_line)
    while i < n:
        c = cmd_line[i]

        if escaped:
            current.append(c)
            escaped = False
            i += 1
            continue

        if c == "\\" and not in_single:
            escaped = True
            current.append(c)
            i += 1
            continue

        if c == "'" and not in_double:
            in_single = not in_single
            current.append(c)
            i += 1
            continue

        if c == '"' and not in_single:
            in_double = not in_double
            current.append(c)
            i += 1
            continue

        if not in_single and not in_double:
            if c in ("\n", ";"):
                segments.append("".join(current).strip())
                current = []
                i += 1
                continue
            if c == "&" and i + 1 < n and cmd_line[i + 1] == "&":
                segments.append("".join(current).strip())
                current = []
                i += 2
                continue
            if c == "|" and i + 1 < n and cmd_line[i + 1] == "|":
                segments.append("".join(current).strip())
                current = []
                i += 2
                continue
            if c in ("|", "&"):
                segments.append("".join(current).strip())
                current = []
                i += 1
                continue

        current.append(c)
        i += 1

    if current:
        segments.append("".join(current).strip())

    return [s for s in segments if s]


def unwrap_command(cmd_line: str) -> list[str]:
    """Recursively unwraps shells (zsh, bash, sh) and splits pipelines/chains."""
    cmd = cmd_line.strip()
    if not cmd:
        return []

    # Check if wrapped in zsh/bash/sh -c
    match = re.match(r"^(?:zsh|bash|sh)\s+-[a-zA-Z]*c\s+(.*)$", cmd, re.DOTALL)
    if match:
        inner = match.group(1).strip()
        # Strip outer quotes if any
        if (inner.startswith("'") and inner.endswith("'")) or (inner.startswith('"') and inner.endswith('"')):
            inner = inner[1:-1]
        return unwrap_command(inner)

    # Split pipelines and command chains
    sub_commands = split_shell_pipeline(cmd)
    if len(sub_commands) > 1:
        unwrapped = []
        for sub in sub_commands:
            unwrapped.extend(unwrap_command(sub))
        return unwrapped

    return [cmd]


def evaluate_git_command(cmd: str) -> tuple[bool, str]:
    """Evaluates whether a git command is destructive."""
    for pattern in DESTRUCTIVE_GIT_PATTERNS:
        if re.search(pattern, cmd, re.IGNORECASE):
            return True, f"Potentially destructive git command: {pattern}"
    return False, "Safe git operation"


def evaluate_adb_command(tokens: list[str]) -> tuple[bool, str]:
    """Evaluates whether an ADB command is destructive."""
    # Strip ADB global options to find the subcommand
    # adb [-d | -e | -s <serial> | -t <id> | -H <host> | -P <port> | -L <socket>] <command>
    i = 1
    while i < len(tokens):
        arg = tokens[i]
        if arg in ("-s", "-t", "-H", "-P", "-L"):
            i += 2
        elif arg.startswith(("-s", "-t", "-H", "-P", "-L")):
            i += 1
        elif arg in ("-d", "-e"):
            i += 1
        else:
            break

    if i >= len(tokens):
        return False, "ADB command without subcommand (help/devices)"

    subcmd = tokens[i]
    rest = tokens[i + 1:]

    # Check top-level ADB destructive subcommands
    if subcmd in DESTRUCTIVE_ADB_SUBCOMMANDS:
        return True, f"Destructive ADB action: 'adb {subcmd}'"

    # Check `adb shell ...`
    if subcmd in ("shell", "exec-out"):
        if not rest:
            return False, "Interactive ADB shell"

        # Combine rest of shell command
        shell_cmd_str = " ".join(rest)
        # Parse shell sub-tokens
        shell_subs = split_shell_pipeline(shell_cmd_str)
        for ssub in shell_subs:
            try:
                stokens = shlex.split(ssub)
            except Exception:
                stokens = ssub.split()
            if not stokens:
                continue

            # Check if running under run-as <package>
            if stokens[0] == "run-as" and len(stokens) > 2:
                stokens = stokens[2:]

            first_word = stokens[0].split("/")[-1]
            if first_word in DESTRUCTIVE_ADB_SHELL_COMMANDS:
                return True, f"Destructive command in ADB shell: '{first_word}'"

            # Check `pm clear` or `pm uninstall`
            if first_word == "pm" and len(stokens) > 1:
                pm_sub = stokens[1]
                if pm_sub in ("clear", "uninstall", "disable", "disable-user"):
                    return True, f"Destructive ADB pm action: 'pm {pm_sub}'"

            # Check `cmd package uninstall`
            if first_word == "cmd" and len(stokens) > 2 and stokens[1] == "package":
                if stokens[2] in ("uninstall", "clear"):
                    return True, f"Destructive ADB cmd package action: '{stokens[2]}'"

    return False, "Safe ADB command"


def evaluate_python_code(code: str) -> tuple[bool, str]:
    """Inspects Python code string for destructive calls or system executions."""
    try:
        tree = ast.parse(code)
    except Exception:
        # Fallback to regex check if code isn't complete valid Python
        for call_name in DESTRUCTIVE_PYTHON_CALLS:
            if re.search(r"\b" + call_name + r"\s*\(", code):
                return True, f"Destructive Python call pattern: '{call_name}()'"
        return False, "Python code could not be parsed as AST, no destructive patterns found"

    for node in ast.walk(tree):
        # Check function / method calls
        if isinstance(node, ast.Call):
            func_name = None
            if isinstance(node.func, ast.Name):
                func_name = node.func.id
            elif isinstance(node.func, ast.Attribute):
                func_name = node.func.attr

            if func_name in DESTRUCTIVE_PYTHON_CALLS:
                return True, f"Destructive Python standard library call: '{func_name}()'"

            # Check os.system, subprocess calls
            if func_name in ("system", "run", "Popen", "call", "check_call", "check_output"):
                # If argument is a string literal, inspect the inner command
                if node.args and isinstance(node.args[0], ast.Constant) and isinstance(node.args[0].value, str):
                    inner_dest, inner_reason = is_command_destructive(node.args[0].value)
                    if inner_dest:
                        return True, f"Destructive command inside Python subprocess: {inner_reason}"

    return False, "Python inline code is non-destructive"


def evaluate_python_command(tokens: list[str]) -> tuple[bool, str]:
    """Evaluates whether a Python/pip/uv command is destructive."""
    cmd_binary = tokens[0].split("/")[-1]

    # Handle uv invocations
    if cmd_binary == "uv":
        if len(tokens) > 1 and tokens[1] == "run":
            tokens = tokens[2:]
            if not tokens:
                return False, "Incomplete uv run"
            cmd_binary = tokens[0].split("/")[-1]
        elif len(tokens) > 2 and tokens[1] == "pip" and tokens[2] == "uninstall":
            return True, "Destructive package removal: uv pip uninstall"
        else:
            return False, "Safe uv operation"

    # Check pip uninstall
    if cmd_binary in ("pip", "pip3"):
        if len(tokens) > 1 and tokens[1] == "uninstall":
            return True, "Destructive package removal: pip uninstall"
        return False, "Safe pip operation"

    # If executing python -c "..."
    if cmd_binary.startswith("python"):
        rest = tokens[1:]
        if "-c" in rest:
            idx = rest.index("-c")
            if idx + 1 < len(rest):
                code = rest[idx + 1]
                return evaluate_python_code(code)

    return False, "Safe Python operation"


def is_command_destructive(cmd_line: str) -> tuple[bool, str]:
    """Evaluates whether a command line is destructive across Zsh, ADB, Python, and system binaries."""
    if not cmd_line or not cmd_line.strip():
        return False, "Empty command"

    # Step 1: Unwrap all shell layers (zsh, bash, sh) and split pipelines/chains
    unwrapped_commands = unwrap_command(cmd_line)

    for cmd in unwrapped_commands:
        cmd = cmd.strip()
        if not cmd:
            continue

        # Check git patterns
        git_dest, git_reason = evaluate_git_command(cmd)
        if git_dest:
            return True, git_reason

        # Check SQL/DB patterns
        for pattern in DESTRUCTIVE_DB_PATTERNS:
            if re.search(pattern, cmd, re.IGNORECASE):
                return True, f"Potentially destructive SQL command: {pattern}"

        # Tokenize command
        try:
            tokens = shlex.split(cmd)
        except Exception:
            tokens = cmd.split()

        if not tokens:
            continue

        binary = tokens[0].split("/")[-1]

        # Check ADB commands
        if binary == "adb":
            adb_dest, adb_reason = evaluate_adb_command(tokens)
            if adb_dest:
                return True, adb_reason
            continue

        # Check Python commands (python, python3, pip, pip3, uv)
        if binary in ("python", "python3", "pip", "pip3") or binary == "uv":
            py_dest, py_reason = evaluate_python_command(tokens)
            if py_dest:
                return True, py_reason
            continue

        # Check standalone destructive system binaries
        if binary in DESTRUCTIVE_SYSTEM_COMMANDS:
            return True, f"Command '{binary}' is considered destructive"

        # Check binary + flag combos (like chmod -R, chown -R)
        if binary in ("chmod", "chown", "chgrp") and any(arg.startswith("-") and "R" in arg for arg in tokens[1:]):
            return True, f"Recursive permission/ownership modification '{binary} -R'"

    return False, "Command verified non-destructive"


def main():
    try:
        raw_input = sys.stdin.read()
        if not raw_input.strip():
            print(json.dumps({"decision": "ask", "reason": "No input provided"}))
            return

        payload = json.loads(raw_input)
        tool_call = payload.get("toolCall", {})
        args = tool_call.get("args", {})
        cmd_line = args.get("CommandLine", "")

        destructive, reason = is_command_destructive(cmd_line)

        if destructive:
            result = {
                "decision": "ask",
                "reason": f"Destructive command detected ({reason}). Confirmation required."
            }
        else:
            result = {
                "decision": "allow",
                "reason": f"Non-destructive command verified and allowed: {reason}."
            }

        print(json.dumps(result))
    except Exception as e:
        # Fallback to ask on any unexpected parsing error for safety
        print(json.dumps({"decision": "ask", "reason": f"Validator error ({e}), falling back to user confirmation."}))


if __name__ == "__main__":
    main()
