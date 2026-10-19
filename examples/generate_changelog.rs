//! Generate a `CHANGELOG.md` release section from the commit history of a branch.
//!
//! The changelog for a release is derived from the commits that landed on the
//! development branch since the previous release tag, so it cannot drift from
//! the history it describes.
//!
//! ```text
//! cargo run --example generate_changelog -- 0.3.2
//! cargo run --example generate_changelog -- 0.3.2 --dry-run
//! cargo run --example generate_changelog -- 0.3.2 --from-tag v0.3.1 --date 2026-10-19
//! ```
//!
//! The example rewrites the `## Unreleased` section of `CHANGELOG.md` in
//! place, replacing it with a `## <version> (<date>)` section. If there is no
//! `Unreleased` section, the new section is inserted after the title.
//!
//! Examples are not installed by `cargo install` and are not included in the
//! release artifacts, so this stays a development-only tool.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::Parser;

/// Repository used to build the pull request and commit links.
const REPO_URL: &str = "https://github.com/brianbirrell/ai-cli";

#[derive(Parser, Debug)]
#[command(
    name = "generate_changelog",
    about = "Generate a CHANGELOG.md release section from the commit history",
    long_about = None,
)]
struct Args {
    /// Release version, for example 0.3.2
    version: String,

    /// Branch to read the history from.
    #[arg(long, value_name = "BRANCH")]
    branch: Option<String>,

    /// Previous release tag to start from.
    #[arg(long, value_name = "TAG")]
    from_tag: Option<String>,

    /// Release date in YYYY-MM-DD format. Defaults to today.
    #[arg(long, value_name = "DATE")]
    date: Option<String>,

    /// Print the generated section without writing CHANGELOG.md.
    #[arg(long)]
    dry_run: bool,
}

/// Changelog section a commit is filed under.
#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
enum Category {
    Security,
    Features,
    Fix,
    Performance,
    Dependencies,
    Documentation,
    SecurityAndCi,
    Refactoring,
    Testing,
    Other,
}

impl Category {
    /// The order categories appear in the generated section.
    const ORDER: [Category; 10] = [
        Category::Security,
        Category::Features,
        Category::Fix,
        Category::Performance,
        Category::Dependencies,
        Category::Documentation,
        Category::SecurityAndCi,
        Category::Refactoring,
        Category::Testing,
        Category::Other,
    ];

    fn heading(self) -> &'static str {
        match self {
            Category::Security => "Security",
            Category::Features => "Features",
            Category::Fix => "Fix",
            Category::Performance => "Performance",
            Category::Dependencies => "Dependencies",
            Category::Documentation => "Documentation",
            Category::SecurityAndCi => "Security and CI",
            Category::Refactoring => "Refactoring",
            Category::Testing => "Testing",
            Category::Other => "Other Changes",
        }
    }
}

/// A commit that is a candidate for the changelog.
struct Commit {
    sha: String,
    subject: String,
    body: String,
}

/// The conventional commit prefix and description of a subject.
struct Subject {
    ctype: Option<String>,
    scope: Option<String>,
    description: String,
}

/// Split `type(scope): description` into its parts.
fn split_subject(subject: &str) -> Subject {
    let trimmed = subject.trim();

    // The prefix is everything before the first ": " that is not inside a
    // scope, which only ever contains characters other than ':'.
    if let Some(colon) = trimmed.find(':') {
        let prefix = &trimmed[..colon];
        let rest = trimmed[colon + 1..].trim_start();

        // A conventional prefix is a single word, optionally with a scope.
        let (ctype, scope) = match prefix.find('(') {
            Some(open) => {
                let name = &prefix[..open];
                let scope = prefix[open + 1..].strip_suffix(')');
                if scope.is_some_and(|s| s.contains(')')) {
                    return fallback_subject(trimmed);
                }
                (name.to_string(), scope.map(str::to_string))
            }
            None => {
                if prefix.is_empty() || !prefix.chars().all(|c| c.is_ascii_alphabetic()) {
                    return fallback_subject(trimmed);
                }
                (prefix.to_string(), None)
            }
        };

        if !rest.is_empty() {
            return Subject {
                ctype: Some(ctype),
                scope,
                description: rest.to_string(),
            };
        }
    }

    fallback_subject(trimmed)
}

/// A subject that does not follow the conventional commit format.
fn fallback_subject(subject: &str) -> Subject {
    Subject {
        ctype: None,
        scope: None,
        description: subject.to_string(),
    }
}

/// Release plumbing that does not belong in a user facing changelog.
fn is_release_plumbing(description: &str) -> bool {
    let lower = description.to_ascii_lowercase();
    const PREFIXES: [&str; 9] = [
        "bump version",
        "update version",
        "update ai-cli version",
        "update changelog.md",
        "updated changelog.md",
        "update cargo.lock",
        "updated cargo.lock",
        "update readme.md",
        "update cargo.toml",
    ];
    if PREFIXES.iter().any(|p| lower.starts_with(p)) {
        return true;
    }
    lower.starts_with("prepare v") && is_version_like(&lower)
}

/// Does the text look like a version number, for example `v0.3.1`?
fn is_version_like(text: &str) -> bool {
    let rest = text.trim_start_matches("prepare v");
    let mut parts = rest.split('.');
    for _ in 0..3 {
        match parts.next() {
            Some(part) if !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()) => {}
            _ => return false,
        }
    }
    parts.next().is_none()
}

/// The category a commit belongs under, or `None` when it should be skipped.
fn classify(subject: &Subject) -> Option<Category> {
    if is_release_plumbing(&subject.description) {
        return None;
    }

    if let Some(scope) = &subject.scope {
        match scope.to_ascii_lowercase().as_str() {
            // Release mechanics are not user facing.
            "release" => return None,
            "deps" | "dep" | "dependencies" => return Some(Category::Dependencies),
            "ci" | "workflow" | "workflows" => return Some(Category::SecurityAndCi),
            _ => {}
        }
    }

    let Some(ctype) = &subject.ctype else {
        return Some(Category::Other);
    };

    match ctype.to_ascii_lowercase().as_str() {
        "feat" | "feature" => Some(Category::Features),
        "fix" => Some(Category::Fix),
        "perf" => Some(Category::Performance),
        "docs" => Some(Category::Documentation),
        "refactor" => Some(Category::Refactoring),
        "test" => Some(Category::Testing),
        "build" => Some(Category::Dependencies),
        "ci" => Some(Category::SecurityAndCi),
        "security" => Some(Category::Security),
        // Plain chores are tooling changes, grouped with CI.
        "chore" => Some(Category::SecurityAndCi),
        _ => Some(Category::Other),
    }
}

/// Does the commit mention a security advisory?
fn mentions_security(commit: &Commit) -> bool {
    let text = format!("{} {}", commit.subject, commit.body).to_ascii_lowercase();
    const HINTS: [&str; 10] = [
        "security",
        "advisory",
        "advisories",
        "vulnerabilit",
        "denial of service",
        "unsound",
        "cve-",
        "ghsa-",
        "exploit",
        "malicious",
    ];
    HINTS.iter().any(|hint| text.contains(hint))
}

/// Tidy up a description for display in the changelog.
fn clean_description(description: &str) -> String {
    let mut text = description.split_whitespace().collect::<Vec<_>>().join(" ");

    // Drop a trailing pull request reference, it is added back as a link.
    if let Some(index) = text.rfind("(#") {
        let rest = &text[index + 2..];
        if rest.ends_with(')') && rest[..rest.len() - 1].chars().all(|c| c.is_ascii_digit()) {
            text = text[..index]
                .trim_end()
                .trim_end_matches([',', ';'])
                .to_string();
        }
    }

    let text = text.trim_end_matches([' ', ',', ';', '.']).to_string();
    if text.is_empty() {
        return text;
    }

    // Capitalise the first letter unless it looks like an identifier.
    let first = text.split(' ').next().unwrap_or_default();
    if first.starts_with(|c: char| c.is_ascii_lowercase()) && !first.contains('_') {
        let mut chars = text.chars();
        let capitalised = chars.next().unwrap().to_ascii_uppercase();
        let mut result = String::with_capacity(text.len());
        result.push(capitalised);
        result.push_str(chars.as_str());
        return result;
    }

    text
}

/// The pull request number referenced by the commit, if any.
fn find_pr(commit: &Commit) -> Option<String> {
    if let Some(index) = commit.subject.rfind("(#") {
        let rest = &commit.subject[index + 2..];
        if let Some(end) = rest.find(')') {
            let digits = &rest[..end];
            if !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()) {
                return Some(digits.to_string());
            }
        }
    }

    if let Some(index) = commit.body.find("/pull/") {
        let rest = &commit.body[index + "/pull/".len()..];
        let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        if !digits.is_empty() {
            return Some(digits);
        }
    }

    None
}

/// Run a git command and return its output.
fn run_git(args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(|error| format!("failed to run git: {error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let detail = if stderr.trim().is_empty() {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        } else {
            stderr.trim().to_string()
        };
        return Err(format!("git {} failed: {detail}", args.join(" ")));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// The repository root, so the tool works from any subdirectory.
fn repo_root() -> Result<PathBuf, String> {
    let output = run_git(&["rev-parse", "--show-toplevel"])?;
    Ok(PathBuf::from(output.trim()))
}

/// The most recent version tag, ignoring development pre-release tags.
fn previous_release_tag() -> Option<String> {
    let output = run_git(&["tag", "-l", "v[0-9]*", "--sort=-v:refname"]).ok()?;
    output.split_whitespace().next().map(str::to_string)
}

/// Resolve the branch to read the history from.
fn resolve_ref(explicit: Option<&str>) -> Result<String, String> {
    if let Some(value) = explicit {
        return Ok(value.to_string());
    }
    for candidate in ["origin/development", "development"] {
        if run_git(&["rev-parse", "--verify", candidate]).is_ok() {
            return Ok(candidate.to_string());
        }
    }
    Err("could not resolve a development branch to read history from".to_string())
}

/// The commits in the range, oldest first.
fn collect_commits(base: Option<&str>, branch: &str) -> Result<Vec<Commit>, String> {
    let revision = match base {
        Some(tag) => format!("{tag}..{branch}"),
        None => branch.to_string(),
    };

    let raw = run_git(&[
        "log",
        "--no-merges",
        "--reverse",
        "--format=%H%x1f%s%x1f%b%x1e",
        &revision,
    ])?;

    let mut commits = Vec::new();
    for record in raw.split('\u{1e}') {
        let record = record.trim_matches('\n');
        if record.trim().is_empty() {
            continue;
        }
        let mut fields = record.split('\u{1f}');
        let sha = fields.next().unwrap_or_default().trim().to_string();
        let subject = fields.next().unwrap_or_default().trim().to_string();
        let body = fields.next().unwrap_or_default().to_string();
        if sha.is_empty() {
            continue;
        }
        commits.push(Commit { sha, subject, body });
    }

    Ok(commits)
}

/// Build the release section from the commits.
fn build_section(version: &str, date: &str, commits: &[Commit]) -> String {
    let mut grouped: BTreeMap<Category, Vec<String>> = BTreeMap::new();

    for commit in commits {
        let subject = split_subject(&commit.subject);
        let Some(category) = classify(&subject) else {
            continue;
        };

        let description = clean_description(&subject.description);
        if description.is_empty() {
            continue;
        }

        let entry = match find_pr(commit) {
            Some(pr) => format!("  - {description} -> [#{pr}]({REPO_URL}/pull/{pr})"),
            None => format!(
                "  - {description} -> [{}]({REPO_URL}/commit/{})",
                &commit.sha[..7.min(commit.sha.len())],
                commit.sha
            ),
        };

        grouped.entry(category).or_default().push(entry.clone());

        // A commit that mentions an advisory is also worth listing under
        // Security, the same way the hand written changelog did.
        if category != Category::Security && mentions_security(commit) {
            grouped.entry(Category::Security).or_default().push(entry);
        }
    }

    let mut section = format!("## {version} ({date})\n");
    for category in Category::ORDER {
        let Some(entries) = grouped.get(&category) else {
            continue;
        };
        section.push('\n');
        section.push_str(category.heading());
        section.push_str(":\n");
        for entry in entries {
            section.push_str(entry);
            section.push('\n');
        }
    }

    section
}

/// Replace the `Unreleased` section, or insert the section before the first
/// existing release section.
fn replace_unreleased(text: &str, section: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let new_section = section.trim_end();

    let Some(start) = lines.iter().position(|line| line.trim() == "## Unreleased") else {
        // No Unreleased section: insert the new section before the first
        // existing release section.
        let mut out: Vec<String> = Vec::with_capacity(lines.len() + 2);
        let mut inserted = false;
        for line in &lines {
            if !inserted && line.starts_with("## ") {
                out.push(new_section.to_string());
                out.push(String::new());
                inserted = true;
            }
            out.push((*line).to_string());
        }
        if !inserted {
            if !out.is_empty() && !out.last().is_some_and(|l| l.is_empty()) {
                out.push(String::new());
            }
            out.push(new_section.to_string());
        }
        return out.join("\n");
    };

    // The section runs until the next heading.
    let end = lines
        .iter()
        .enumerate()
        .skip(start + 1)
        .find(|(_, line)| line.starts_with("## "))
        .map(|(index, _)| index)
        .unwrap_or(lines.len());

    let mut out: Vec<String> = lines[..start]
        .iter()
        .map(|line| (*line).to_string())
        .collect();
    out.push(new_section.to_string());
    out.push(String::new());
    out.extend(lines[end..].iter().map(|line| (*line).to_string()));
    out.join("\n")
}

/// Today, in UTC, as `YYYY-MM-DD`.
fn today_utc() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0);
    let (year, month, day) = civil_from_days(seconds / 86_400);
    format!("{year:04}-{month:02}-{day:02}")
}

/// Convert a count of days since the Unix epoch into a calendar date.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let shifted = days + 719_468;
    let era = if shifted >= 0 {
        shifted
    } else {
        shifted - 146_096
    } / 146_097;
    let day_of_era = (shifted - era * 146_097) as u64;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36524 - day_of_era / 146_096) / 365;
    let year = year_of_era as i64 + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = (day_of_year - (153 * month_prime + 2) / 5 + 1) as u32;
    let month = if month_prime < 10 {
        month_prime + 3
    } else {
        month_prime - 9
    } as u32;
    (if month <= 2 { year + 1 } else { year }, month, day)
}

/// Is the text a `MAJOR.MINOR.PATCH` version?
fn is_valid_version(version: &str) -> bool {
    let mut parts = version.split('.');
    for _ in 0..3 {
        match parts.next() {
            Some(part) if !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()) => {}
            _ => return false,
        }
    }
    parts.next().is_none()
}

fn main() -> Result<(), String> {
    let args = Args::parse();

    if !is_valid_version(&args.version) {
        return Err(format!(
            "invalid version '{}', expected MAJOR.MINOR.PATCH",
            args.version
        ));
    }

    let branch = resolve_ref(args.branch.as_deref())?;
    let base = args.from_tag.clone().or_else(previous_release_tag);
    let date = args.date.clone().unwrap_or_else(today_utc);

    let commits = collect_commits(base.as_deref(), &branch)?;
    if commits.is_empty() {
        return Err(format!(
            "no commits found in {}..{branch}",
            base.as_deref().unwrap_or("root")
        ));
    }

    let section = build_section(&args.version, &date, &commits);

    if !section.lines().any(|line| line.starts_with("  - ")) {
        return Err(format!(
            "no changelog entries were generated from {}..{branch}. \
             Check the range, or pass --from-tag if the previous release tag is wrong.",
            base.as_deref().unwrap_or("root")
        ));
    }

    if args.dry_run {
        print!("{section}");
        return Ok(());
    }

    let changelog = repo_root()?.join("CHANGELOG.md");
    let original = std::fs::read_to_string(&changelog)
        .map_err(|error| format!("failed to read {}: {error}", changelog.display()))?;

    if original.lines().any(|line| {
        line.trim_start()
            .starts_with(&format!("## {}", args.version))
    }) {
        return Err(format!(
            "CHANGELOG.md already has a '{}' section. Refusing to create a duplicate.",
            args.version
        ));
    }

    let updated = replace_unreleased(&original, &section);
    std::fs::write(&changelog, updated)
        .map_err(|error| format!("failed to write {}: {error}", changelog.display()))?;

    println!(
        "Updated CHANGELOG.md: {} commit(s) from {}..{branch}",
        commits.len(),
        base.as_deref().unwrap_or("root")
    );
    println!("  release section: {} ({date})", args.version);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn subject_of(text: &str) -> Subject {
        split_subject(text)
    }

    fn category_of(text: &str) -> Option<Category> {
        classify(&subject_of(text))
    }

    #[test]
    fn splits_conventional_subjects() {
        let subject = subject_of("feat(deb): add support for building deb packages");
        assert_eq!(subject.ctype.as_deref(), Some("feat"));
        assert_eq!(subject.scope.as_deref(), Some("deb"));
        assert_eq!(subject.description, "add support for building deb packages");
    }

    #[test]
    fn splits_plain_subjects() {
        let subject = subject_of("Update dependabot configuration");
        assert_eq!(subject.ctype, None);
        assert_eq!(subject.scope, None);
        assert_eq!(subject.description, "Update dependabot configuration");
    }

    #[test]
    fn classifies_by_type() {
        assert_eq!(category_of("feat: add a feature"), Some(Category::Features));
        assert_eq!(category_of("fix: correct a bug"), Some(Category::Fix));
        assert_eq!(
            category_of("docs: update the readme"),
            Some(Category::Documentation)
        );
        assert_eq!(
            category_of("refactor: tidy up"),
            Some(Category::Refactoring)
        );
        assert_eq!(category_of("test: add a test"), Some(Category::Testing));
        assert_eq!(
            category_of("perf: make it faster"),
            Some(Category::Performance)
        );
    }

    #[test]
    fn scope_overrides_type() {
        assert_eq!(
            category_of("chore(deps): bump reqwest"),
            Some(Category::Dependencies)
        );
        assert_eq!(
            category_of("chore(workflow): adjust the workflow"),
            Some(Category::SecurityAndCi)
        );
    }

    #[test]
    fn release_plumbing_is_skipped() {
        assert_eq!(category_of("chore(release): prepare v0.3.1"), None);
        assert_eq!(category_of("chore: update ai-cli version to 0.3.0"), None);
        assert_eq!(category_of("chore: update Cargo.lock"), None);
        assert_eq!(category_of("Update CHANGELOG.md"), None);
        assert_eq!(category_of("chore: bump version to 0.3.1"), None);
    }

    #[test]
    fn plain_chores_are_grouped_with_ci() {
        assert_eq!(
            category_of("chore: format git command arguments"),
            Some(Category::SecurityAndCi)
        );
    }

    #[test]
    fn unknown_types_fall_into_other() {
        // A chore with an unrecognised scope is still a chore, so it is
        // grouped with CI rather than with user facing changes.
        assert_eq!(
            category_of("chore(wibble): something"),
            Some(Category::SecurityAndCi)
        );
        assert_eq!(
            category_of("something entirely different"),
            Some(Category::Other)
        );
    }

    #[test]
    fn cleans_descriptions() {
        assert_eq!(
            clean_description("add support for building deb packages"),
            "Add support for building deb packages"
        );
        assert_eq!(clean_description("fix the thing (#93)"), "Fix the thing");
        assert_eq!(clean_description("  spaced   out  "), "Spaced out");
    }

    #[test]
    fn keeps_identifiers_lowercase() {
        assert_eq!(
            clean_description("bump reqwest to 0.13"),
            "Bump reqwest to 0.13"
        );
        assert_eq!(clean_description("update Cargo.toml"), "Update Cargo.toml");
    }

    #[test]
    fn finds_pull_request_numbers() {
        let commit = Commit {
            sha: "abc1234".to_string(),
            subject: "chore(deps): bump reqwest (#93)".to_string(),
            body: String::new(),
        };
        assert_eq!(find_pr(&commit), Some("93".to_string()));

        let from_body = Commit {
            sha: "abc1234".to_string(),
            subject: "chore(deps): bump reqwest".to_string(),
            body: "See https://github.com/brianbirrell/ai-cli/pull/91".to_string(),
        };
        assert_eq!(find_pr(&from_body), Some("91".to_string()));

        let none = Commit {
            sha: "abc1234".to_string(),
            subject: "chore(deps): bump reqwest".to_string(),
            body: String::new(),
        };
        assert_eq!(find_pr(&none), None);
    }

    #[test]
    fn detects_security_mentions() {
        let advisory = Commit {
            sha: "abc1234".to_string(),
            subject: "chore(deps): bump reqwest".to_string(),
            body: "clears 12 open Dependabot security alerts".to_string(),
        };
        assert!(mentions_security(&advisory));

        let ghsa = Commit {
            sha: "abc1234".to_string(),
            subject: "fix: patch GHSA-abcd-efgh-ijkl".to_string(),
            body: String::new(),
        };
        assert!(mentions_security(&ghsa));

        let plain = Commit {
            sha: "abc1234".to_string(),
            subject: "chore: tidy up the build script".to_string(),
            body: String::new(),
        };
        assert!(!mentions_security(&plain));
    }

    #[test]
    fn groups_security_commits_twice() {
        let commit = Commit {
            sha: "abcdef1234".to_string(),
            subject: "chore(deps): bump reqwest".to_string(),
            body: "clears open security alerts".to_string(),
        };
        let section = build_section("0.3.2", "2026-10-19", &[commit]);
        assert!(section.contains("Security:"));
        assert!(section.contains("Dependencies:"));
    }

    #[test]
    fn replaces_the_unreleased_section() {
        let original =
            "# CHANGELOG.md\n\n## Unreleased\n\nold stuff\n\n## 0.3.0 (2026-03-23)\n\nkept\n";
        let updated = replace_unreleased(original, "## 0.3.2 (2026-10-19)\n\nnew\n");
        assert!(updated.contains("## 0.3.2 (2026-10-19)"));
        assert!(!updated.contains("Unreleased"));
        assert!(updated.contains("## 0.3.0 (2026-03-23)"));
        assert!(updated.contains("kept"));
    }

    #[test]
    fn inserts_when_there_is_no_unreleased_section() {
        let original = "# CHANGELOG.md\n\n## 0.3.0 (2026-03-23)\n\nkept\n";
        let updated = replace_unreleased(original, "## 0.3.2 (2026-10-19)\n\nnew\n");
        assert!(updated.starts_with("# CHANGELOG.md\n\n## 0.3.2 (2026-10-19)"));
        assert!(updated.contains("## 0.3.0 (2026-03-23)"));
        assert!(updated.contains("kept"));
    }

    #[test]
    fn validates_versions() {
        assert!(is_valid_version("0.3.2"));
        assert!(is_valid_version("10.20.30"));
        assert!(!is_valid_version("0.3"));
        assert!(!is_valid_version("0.3.2.1"));
        assert!(!is_valid_version("v0.3.2"));
        assert!(!is_valid_version("0.3.x"));
    }

    #[test]
    fn converts_epoch_days_to_dates() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(19_723), (2024, 1, 1));
        assert_eq!(civil_from_days(20_745), (2026, 10, 19));
    }
}
