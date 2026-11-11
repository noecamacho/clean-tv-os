---
name: zsh-safe-execution
description: >-
  Use this skill to evaluate, structure, and safely run commands in zsh, ADB, and Python,
  distinguishing between non-destructive operations (inspections, builds, diagnostics, installations)
  and potentially destructive actions (file deletions, uninstalls, hard resets, wipes).
---

# Zsh, ADB & Python Safe Execution Skill

This skill guides the evaluation and execution of terminal commands across **Zsh**, **Android Debug Bridge (ADB)**, and **Python**. It ensures that safe, non-destructive commands are automatically allowed by the Antigravity `PreToolUse` lifecycle hook, while dangerous, destructive, or state-wiping commands prompt for explicit user confirmation.

---

## 1. Execution Standard

In accordance with project rules:
* Always execute commands using **`zsh`** with interactive login environment:
  ```bash
  zsh -lc "<command>"
  ```
* This ensures that user environment variables, aliases, and tools like `uv`, Rust, Android SDK, and Go in `PATH` are properly loaded.

---

## 2. Command Evaluation Matrices

### A. Zsh & Shell Commands

The validator automatically unwraps nested shell wrappers (`zsh -lc`, `zsh -c`, `bash -c`, `sh -c`) and evaluates every sub-command across pipelines and chains (`;`, `&&`, `||`, `|`, `&`).

* **Safe / Non-Destructive (ALLOW)**:
  - **Inspection & Reading**: `ls`, `cat`, `head`, `tail`, `grep`, `rg`, `sed -n`, `awk`, `find`, `wc`, `file`, `stat`
  - **Version Control**: `git status`, `git diff`, `git log`, `git branch`, `git show`, `git rev-parse`, `git add`, `git commit`
  - **Rust / Cargo**: `cargo test`, `cargo clippy`, `cargo check`, `cargo fmt`, `cargo run`, `cargo build`
  - **Build & Emulation**: `scripts/container-build.sh`, `scripts/emulator.sh`
  - **OpenSpec Workflow**: `openspec list`, `openspec show`, `openspec status`, `openspec validate`

* **Destructive (ASK / Confirmation Required)**:
  - **Filesystem & Wiping**: `rm`, `unlink`, `rmdir`, `shred`, `wipe`, `srm`, `chmod -R`, `chown -R`
  - **Disk Formatting**: `mkfs`, `fdisk`, `parted`, `dd`
  - **Destructive Git**: `git reset`, `git clean`, `git restore`, `git checkout -- .`, `git push --force`, `git rebase`
  - **System Termination**: `reboot`, `shutdown`, `poweroff`, `halt`, `kill -9`

---

### B. Android Debug Bridge (ADB) Commands

The validator parses global ADB flags (`-s <device>`, `-d`, `-e`, `-H`, `-P`, `-L`) and deeply analyzes subcommands, including shell commands and `run-as` contexts.

* **Safe / Non-Destructive (ALLOW)**:
  - **Diagnostics & Status**: `adb devices`, `adb logcat`, `adb wait-for-device`, `adb get-state`
  - **Activity & System Dump**: `adb shell dumpsys ...`, `adb shell getprop ...`
  - **Input Simulation**: `adb shell input keyevent ...`, `input text ...`, `input tap ...`
  - **App Lifecycle & Navigation**: `adb shell am start ...`, `adb shell am force-stop ...`, `adb shell cmd package set-home-activity ...`
  - **Inspection & Testing**: `adb exec-out screencap ...`, `adb shell pm list ...`, `adb shell pm path ...`
  - **File Transfer & Debug Build Install**: `adb push ...`, `adb pull ...`, `adb install ...`, `adb install -r ...`
  - **App Sandbox Inspection**: `adb shell run-as <pkg> cat ...`, `adb shell run-as <pkg> cp ...`

* **Destructive (ASK / Confirmation Required)**:
  - **Removal & Data Wipe**: `adb uninstall <pkg>`, `adb shell pm clear <pkg>`, `adb shell pm uninstall <pkg>`
  - **Shell Deletion & Formatting**: `adb shell rm ...`, `adb shell wipe ...`, `adb shell dd ...`, `adb shell mkfs ...`
  - **System State & Boot**: `adb reboot ...`, `adb shell reboot ...`, `adb shell recovery`, `adb shell bootloader`
  - **Privilege & System Partition**: `adb remount`, `adb root`, `adb unroot`, `adb shell su ...`, `adb shell setprop ...`
  - **Component Disabling**: `adb shell pm disable ...`, `adb shell pm disable-user ...`

---

### C. Python & Tooling Commands

The validator inspects direct invocations (`python`, `python3`), virtual environment tooling (`uv run ...`), and inline code (`-c "..."`) using AST and lexical analysis.

* **Safe / Non-Destructive (ALLOW)**:
  - **Repository Analysis Scripts**: `uv run scripts/measure-ink.py ...`, `uv run scripts/verify-apk.py ...`, `scripts/make-test-fonts.py`
  - **Inline Code**: String parsing, JSON serialization (`json.loads`, `json.dumps`), math, image processing, measurements.
  - **Dependency Management**: `pip install ...`, `uv pip install ...`, `uv add ...`

* **Destructive (ASK / Confirmation Required)**:
  - **Filesystem Removal Calls**: `os.remove()`, `os.unlink()`, `os.rmdir()`, `os.removedirs()`, `shutil.rmtree()`, `pathlib.Path.unlink()`
  - **Subprocess Shell Calls**: `os.system("...")`, `subprocess.run(["..."])` containing destructive commands
  - **Package Removal**: `pip uninstall ...`, `uv pip uninstall ...`

---

## 3. How the Gatekeeper Works

This skill operates in synergy with the workspace Lifecycle Hook configured in [`.agents/hooks.json`](../hooks.json):
1. **Tool Interception**: Before `run_command` executes, Antigravity triggers `PreToolUse`.
2. **Payload Inspection**: [`.agents/scripts/validate-safe-command.py`](../scripts/validate-safe-command.py) parses the `CommandLine` argument from `stdin`.
3. **Automated Decision**:
   - Returns `{"decision": "allow", "reason": "..."}` for non-destructive operations.
   - Returns `{"decision": "ask", "reason": "..."}` for destructive operations to prompt the user.

---

## 4. Best Practices for Pair Programming

1. **Keep Commands Targeted**: Avoid mixing safe diagnostics with destructive actions in single chained strings.
2. **Prefer Explicit Subcommands**: Clearly specify the target device (`adb -s <ip>:5555`) and direct subcommands.
3. **Preserve Validation Transparency**: When a command prompts for confirmation, review the exact destructive pattern identified before granting approval.
