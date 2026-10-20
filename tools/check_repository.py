"""Review Git candidate bytes without printing suspected secret values (Python 3.9+)."""

import argparse
from pathlib import Path, PurePosixPath
import re
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[1]
PRIVATE_DIRECTORIES = {
    ".aws", ".azure", ".ssh", ".codex", ".agents", ".secrets", "secrets",
    "credentials", "local-data", ".p0-tools", "target", "artifacts", "__pycache__",
    ".venv", "venv", "node_modules", ".pytest_cache", ".docker",
}
PRIVATE_SUFFIXES = {".pem", ".key", ".p12", ".pfx", ".jks", ".log", ".tmp", ".temp", ".bak", ".pyc"}
SECRET_PATTERNS = (
    ("private-key", re.compile(rb"-----BEGIN (?:[A-Z0-9]+ )*PRIVATE KEY-----")),
    ("aws-access-key", re.compile(rb"\b(?:AKIA|ASIA)[A-Z0-9]{16}\b")),
    ("github-token", re.compile(rb"\b(?:gh[pousr]_[A-Za-z0-9]{30,}|github_pat_[A-Za-z0-9_]{40,})\b")),
    ("slack-token", re.compile(rb"\bxox[baprs]-[A-Za-z0-9-]{20,}\b")),
    ("openai-token", re.compile(rb"\bsk-(?:proj-|svcacct-)?[A-Za-z0-9_-]{40,}\b")),
    ("credential-url", re.compile(rb"[a-zA-Z][a-zA-Z0-9+.-]*://[^\s/:@]+:[^\s/@]{4,}@")),
    ("secret-assignment", re.compile(
        rb"(?i)(?:[\"']?(?:api[_-]?key|client[_-]?secret|access[_-]?token|password|passwd|authorization)[\"']?)"
        rb"\s*[:=]\s*[\"']([^\"'\r\n]{12,})[\"']"
    )),
)


def path_issues(relative):
    path = PurePosixPath(relative)
    parts = [part.lower() for part in path.parts]
    name = parts[-1] if parts else ""
    issues = []
    if path.is_absolute() or ".." in parts or "\\" in relative:
        issues.append("invalid-repository-path")
    if any(part in PRIVATE_DIRECTORIES for part in parts):
        issues.append("private-or-generated-directory")
    if name in {".env", ".envrc", ".netrc", ".npmrc", ".pypirc"} or name.startswith(".env.") or name.endswith(".env"):
        issues.append("environment-file")
    if (PurePosixPath(name).suffix in PRIVATE_SUFFIXES or ".local." in name
            or name.endswith(".local") or name.startswith(("id_rsa", "id_ed25519", "service-account"))
            or name in {"credentials.json", "thumbs.db", ".ds_store"}):
        issues.append("private-or-generated-file")
    return issues


def content_issues(data):
    findings = []
    for rule, pattern in SECRET_PATTERNS:
        for match in pattern.finditer(data):
            if rule == "secret-assignment":
                value = match.group(1).lower()
                if any(marker in value for marker in (b"example", b"placeholder", b"your_", b"your-", b"${", b"<", b"redacted")):
                    continue
            findings.append({"rule": rule, "line": data.count(b"\n", 0, match.start()) + 1})
    return findings


def git(*arguments):
    return subprocess.check_output(["git", *arguments], cwd=ROOT)


def inspect(staged=False):
    arguments = (["diff", "--cached", "--name-only", "--diff-filter=ACMR", "-z"] if staged else
                 ["ls-files", "--cached", "--others", "--exclude-standard", "-z"])
    paths = sorted({p.decode("utf-8") for p in git(*arguments).split(b"\0") if p})
    modes = {}
    for record in git("ls-files", "--stage", "-z").split(b"\0"):
        if record:
            header, name = record.split(b"\t", 1)
            modes[name.decode("utf-8")] = header.split()[0].decode("ascii")
    failures = []
    for relative in paths:
        issues = path_issues(relative)
        for rule in issues:
            failures.append((relative, rule, None))
        if "invalid-repository-path" in issues:
            continue
        path = ROOT / relative
        if modes.get(relative) in {"120000", "160000"} or (not staged and path.is_symlink()):
            failures.append((relative, "symlink-or-submodule-requires-review", None))
            continue
        if not staged and not path.is_file():
            if path.exists():
                failures.append((relative, "unsupported-file-kind", None))
            continue
        data = git("show", ":" + relative) if staged else path.read_bytes()
        if len(data) > 20 * 1024 * 1024:
            failures.append((relative, "large-file-requires-explicit-storage-plan", None))
        for finding in content_issues(data):
            failures.append((relative, finding["rule"], finding["line"]))
    return len(paths), failures


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--staged", action="store_true", help="Scan actual index bytes, including ignored tracked files")
    options = parser.parse_args()
    count, failures = inspect(options.staged)
    for relative, rule, line in failures:
        print("REJECT: %s%s [%s]" % (relative, (":" + str(line)) if line else "", rule), file=sys.stderr)
    if failures:
        print("Suspected values are redacted. Review findings; no files were changed.", file=sys.stderr)
        return 1
    print("PASS: repository %s review, %d files; common secret/path checks only, not an exhaustive security or asset-license audit."
          % ("staged" if options.staged else "candidate", count))
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, subprocess.CalledProcessError) as error:
        print("Repository review could not complete: " + str(error), file=sys.stderr)
        sys.exit(1)
