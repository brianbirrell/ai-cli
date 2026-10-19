#!/usr/bin/env python3
"""Generate a CHANGELOG.md release section from the commit history of a branch.

The changelog for a release is derived from the commits that landed on the
development branch since the previous release tag, so it cannot drift from the
history it describes.

Usage:
    python3 scripts/generate_changelog.py 0.3.2
    python3 scripts/generate_changelog.py 0.3.2 --ref origin/development
    python3 scripts/generate_changelog.py 0.3.2 --date 2026-10-19
    python3 scripts/generate_changelog.py 0.3.2 --dry-run

The script rewrites the `## Unreleased` section of CHANGELOG.md in place,
replacing it with a `## <version> (<date>)` section. If there is no
`Unreleased` section, the new section is inserted after the title.

Requires Python 3.8+ and a git working tree.
"""

from __future__ import annotations

import argparse
import datetime as dt
import re
import subprocess
import sys
from collections import OrderedDict
from pathlib import Path

REPO_URL = "https://github.com/brianbirrell/ai-cli"
CHANGELOG = Path(__file__).resolve().parent.parent / "CHANGELOG.md"

# Section order in the generated output.
CATEGORY_ORDER = [
    "Security",
    "Features",
    "Fix",
    "Performance",
    "Dependencies",
    "Documentation",
    "Security and CI",
    "Refactoring",
    "Testing",
    "Other Changes",
]

# Conventional commit type -> changelog category.
TYPE_TO_CATEGORY = {
    "feat": "Features",
    "feature": "Features",
    "fix": "Fix",
    "perf": "Performance",
    "docs": "Documentation",
    "refactor": "Refactoring",
    "test": "Testing",
    "build": "Dependencies",
    "ci": "Security and CI",
    "security": "Security",
}

# Scope overrides, checked before the type mapping.
SCOPE_TO_CATEGORY = {
    "deps": "Dependencies",
    "dep": "Dependencies",
    "dependencies": "Dependencies",
    "release": None,  # release mechanics are not user facing
    "ci": "Security and CI",
    "workflow": "Security and CI",
    "workflows": "Security and CI",
}

# Commits that are release plumbing and do not belong in a user facing changelog.
# Matched against the description, with the conventional commit prefix removed.
SKIP_DESCRIPTION = re.compile(
    r"^(bump version|update (ai-cli )?version\b|update changelog\.md|updated changelog\.md"
    r"|update cargo\.lock|updated cargo\.lock|update readme\.md|update cargo\.toml"
    r"|prepare v?\d+\.\d+\.\d+)",
    re.IGNORECASE,
)

# A commit that mentions any of these is also listed under Security.
SECURITY_HINT = re.compile(
    r"\b(security|advisory|advisories|CVE-\d{4}-\d+|GHSA-[a-z0-9]{4}-[a-z0-9]{4}-[a-z0-9]{4}"
    r"|vulnerabilit(y|ies)|denial of service|unsound)\b",
    re.IGNORECASE,
)

CONVENTIONAL = re.compile(
    r"^(?P<type>[a-zA-Z]+)(?:\((?P<scope>[^)]+)\))?(?P<breaking>!)?:\s*(?P<desc>.+)$"
)
PR_IN_SUBJECT = re.compile(r"\(#(?P<num>\d+)\)")
PR_IN_BODY = re.compile(r"/pull/(?P<num>\d+)")

# Sentence-case the description but leave identifiers and proper nouns alone.
def clean_description(text: str) -> str:
    text = " ".join(text.split())
    text = PR_IN_SUBJECT.sub("", text).strip()
    text = re.sub(r"[\s,;.]+$", "", text)
    if not text:
        return text
    # Capitalise the first letter unless the word looks like an identifier.
    first = text.split(" ", 1)[0]
    if first and first[0].islower() and not re.match(r"^[a-z0-9_.\-]*[A-Z_]", first):
        text = text[0].upper() + text[1:]
    return text


def run_git(*args: str) -> str:
    result = subprocess.run(
        ["git", *args], capture_output=True, text=True, check=False
    )
    if result.returncode != 0:
        raise RuntimeError(
            f"git {' '.join(args)} failed: {result.stderr.strip() or result.stdout.strip()}"
        )
    return result.stdout


def previous_release_tag() -> str | None:
    """The most recent version tag, ignoring development pre-release tags."""
    tags = run_git("tag", "-l", "v[0-9]*", "--sort=-v:refname").split()
    return tags[0] if tags else None


def resolve_ref(ref: str | None) -> str:
    if ref:
        return ref
    for candidate in ("origin/development", "development"):
        check = subprocess.run(
            ["git", "rev-parse", "--verify", candidate],
            capture_output=True,
            text=True,
            check=False,
        )
        if check.returncode == 0:
            return candidate
    raise RuntimeError("could not resolve a development branch to read history from")


def collect_commits(base: str | None, ref: str) -> list[tuple[str, str, str]]:
    """Return (sha, subject, body) for commits in the range, oldest first."""
    revision = f"{base}..{ref}" if base else ref
    raw = run_git(
        "log",
        "--no-merges",
        "--reverse",
        "--format=%H%x1f%s%x1f%b%x1e",
        revision,
    )
    commits: list[tuple[str, str, str]] = []
    for record in raw.split("\x1e"):
        record = record.strip("\n")
        if not record.strip():
            continue
        parts = record.split("\x1f")
        if len(parts) < 3:
            continue
        commits.append((parts[0].strip(), parts[1].strip(), parts[2]))
    return commits


def split_subject(subject: str) -> tuple[str | None, str | None, str]:
    """Return (type, scope, description) for a conventional commit subject."""
    match = CONVENTIONAL.match(subject)
    if not match:
        return None, None, subject
    return (
        match.group("type").lower(),
        (match.group("scope") or "").lower(),
        match.group("desc"),
    )


def classify(subject: str) -> str | None:
    """Map a commit subject to a changelog category, or None to skip it."""
    ctype, scope, description = split_subject(subject)

    if SKIP_DESCRIPTION.match(description):
        return None

    if scope in SCOPE_TO_CATEGORY:
        # May be None, which means the commit is release plumbing.
        return SCOPE_TO_CATEGORY[scope]

    if ctype is None:
        return "Other Changes"

    if ctype == "chore":
        # Plain chores are tooling changes; group them with CI.
        return "Security and CI"

    return TYPE_TO_CATEGORY.get(ctype, "Other Changes")


def find_pr(subject: str, body: str) -> str | None:
    match = PR_IN_SUBJECT.search(subject)
    if match:
        return match.group("num")
    match = PR_IN_BODY.search(body)
    if match:
        return match.group("num")
    return None


def build_section(version: str, date: str, commits: list[tuple[str, str, str]]) -> str:
    grouped: "OrderedDict[str, list[str]]" = OrderedDict()

    for sha, subject, body in commits:
        category = classify(subject)
        if category is None:
            continue

        _, _, description = split_subject(subject)
        description = clean_description(description)
        if not description:
            continue

        pr = find_pr(subject, body)
        if pr:
            entry = f"  - {description} -> [#{pr}]({REPO_URL}/pull/{pr})"
        else:
            entry = f"  - {description} -> [{sha[:7]}]({REPO_URL}/commit/{sha})"

        grouped.setdefault(category, []).append(entry)

        # A commit that mentions an advisory is also worth listing under
        # Security, the same way the hand written changelog did.
        if SECURITY_HINT.search(subject) or SECURITY_HINT.search(body):
            if category != "Security":
                grouped.setdefault("Security", []).append(entry)

    ordered = [c for c in CATEGORY_ORDER if c in grouped]
    for category in grouped:
        if category not in ordered:
            ordered.append(category)

    lines = [f"## {version} ({date})", ""]
    for category in ordered:
        lines.append(f"{category}:")
        lines.extend(grouped[category])
        lines.append("")
    return "\n".join(lines).rstrip() + "\n"


def replace_unreleased(text: str, section: str) -> str:
    """Replace the Unreleased section, or insert the section after the title."""
    pattern = re.compile(r"^## Unreleased\s*$.*?(?=^## |\Z)", re.MULTILINE | re.DOTALL)
    if pattern.search(text):
        return pattern.sub(section, text, count=1)

    # No Unreleased section: insert after the top level title.
    title = re.search(r"^# .+$", text, re.MULTILINE)
    if title:
        end = title.end()
        return text[: end + 1] + "\n" + section + "\n" + text[end + 1 :].lstrip("\n")
    return section + "\n" + text


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("version", help="release version, for example 0.3.2")
    parser.add_argument(
        "--ref",
        default=None,
        help="branch to read history from (default: origin/development)",
    )
    parser.add_argument(
        "--date",
        default=None,
        help="release date, YYYY-MM-DD (default: today)",
    )
    parser.add_argument(
        "--from-tag",
        default=None,
        help="previous release tag (default: the most recent v* tag)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="print the generated section without writing CHANGELOG.md",
    )
    args = parser.parse_args()

    if not re.fullmatch(r"\d+\.\d+\.\d+", args.version):
        print(f"error: invalid version '{args.version}', expected MAJOR.MINOR.PATCH",
              file=sys.stderr)
        return 1

    ref = resolve_ref(args.ref)
    base = args.from_tag or previous_release_tag()
    date = args.date or dt.date.today().isoformat()

    commits = collect_commits(base, ref)
    if not commits:
        print(f"error: no commits found in {base or 'root'}..{ref}", file=sys.stderr)
        return 1

    section = build_section(args.version, date, commits)

    if not any(line.startswith("  - ") for line in section.splitlines()):
        print(
            f"error: no changelog entries were generated from "
            f"{base or 'root'}..{ref}. Refusing to write an empty release "
            f"section. Check the range, or pass --from-tag if the previous "
            f"release tag is wrong.",
            file=sys.stderr,
        )
        return 1

    if args.dry_run:
        print(section)
        return 0

    original = CHANGELOG.read_text()

    if re.search(rf"^## {re.escape(args.version)}(\s|\(|$)", original, re.MULTILINE):
        print(
            f"error: CHANGELOG.md already has a '{args.version}' section. "
            f"Refusing to create a duplicate.",
            file=sys.stderr,
        )
        return 1

    updated = replace_unreleased(original, section)
    CHANGELOG.write_text(updated)

    print(f"Updated {CHANGELOG.name}: {len(commits)} commit(s) from {base or 'root'}..{ref}")
    print(f"  release section: {args.version} ({date})")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except RuntimeError as error:
        print(f"error: {error}", file=sys.stderr)
        sys.exit(1)
