#!/usr/bin/env python3
"""Build a single, reviewable LLM context file from the maintained wiki pages."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PAGES = [
    "INDEX.md",
    "context.md",
    "architecture.md",
    "design.md",
    "operations.md",
    "references.md",
    "next-work.md",
]


def main() -> None:
    wiki = ROOT / "docs" / "llm-wiki"
    sections = [
        "# HOME · contexto compilado para LLM\n",
        "Generado desde docs/llm-wiki/ con uv run python scripts/build-llm-wiki.py. "
        "Edita las páginas fuente y regenera este archivo. No contiene secretos.\n",
    ]
    for name in PAGES:
        sections.append(f"\n---\n\n<!-- source: docs/llm-wiki/{name} -->\n")
        sections.append((wiki / name).read_text(encoding="utf-8").rstrip() + "\n")
    (ROOT / "llms-full.txt").write_text("\n".join(sections), encoding="utf-8")


if __name__ == "__main__":
    main()
