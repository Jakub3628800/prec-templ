#!/usr/bin/env python3
"""Update pinned pre-commit hook revisions in Jinja templates."""

from __future__ import annotations

import re
import subprocess
from pathlib import Path

from packaging.version import InvalidVersion, Version


ROOT = Path(__file__).resolve().parents[1]

HOOK_REPOS = {
    "https://github.com/astral-sh/ruff-pre-commit": "templates/python.j2",
    "https://github.com/astral-sh/uv-pre-commit": "templates/python.j2",
    "https://github.com/pre-commit/pre-commit-hooks": "templates/base.j2",
    "https://github.com/pre-commit/mirrors-prettier": "templates/js.j2",
    "https://github.com/pre-commit/mirrors-eslint": "templates/js.j2",
    "https://github.com/hadolint/hadolint": "templates/docker.j2",
    "https://github.com/rhysd/actionlint": "templates/github_actions.j2",
    "https://github.com/golangci/golangci-lint": "templates/go.j2",
}


def latest_tag(repo: str) -> str:
    output = subprocess.check_output(
        ["git", "ls-remote", "--tags", repo],
        text=True,
    )
    versions: list[tuple[Version, str]] = []
    for line in output.splitlines():
        tag = line.rsplit("/", 1)[-1]
        if tag.endswith("^{}"):
            continue
        normalized = tag.removeprefix("v")
        try:
            version = Version(normalized)
        except InvalidVersion:
            continue
        if version.is_prerelease:
            continue
        versions.append((version, tag))

    if not versions:
        raise RuntimeError(f"No stable semver-like tags found for {repo}")

    return max(versions, key=lambda item: item[0])[1]


def update_template(repo: str, relative_path: str, tag: str) -> bool:
    path = ROOT / relative_path
    text = path.read_text()
    pattern = re.compile(
        rf"(- repo: {re.escape(repo)}\n\s+rev: )([^\n]+)",
        re.MULTILINE,
    )
    updated, count = pattern.subn(rf"\g<1>{tag}", text)
    if count != 1:
        raise RuntimeError(f"Expected one {repo} entry in {relative_path}, found {count}")
    if updated == text:
        return False
    path.write_text(updated)
    return True


def main() -> None:
    changed = False
    for repo, relative_path in HOOK_REPOS.items():
        tag = latest_tag(repo)
        changed = update_template(repo, relative_path, tag) or changed
        print(f"{repo}: {tag}")

    if not changed:
        print("Hook templates are already up to date.")


if __name__ == "__main__":
    main()
