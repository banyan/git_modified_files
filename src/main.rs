use std::path::Path;
use std::process::Command;

const HELP: &str = "Usage: git modified_files [object]

A Git subcommand to list modified files in git commit or from current working tree

Arguments:
  [object]  Commit to list files of (default: the current working tree)

Options:
  -h, --help     Show this help
  -V, --version  Show version";

const BINARY_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "webp", "avif", "gif", "svg", "eot", "mp3", "ttf", "wav", "woff", "woff2",
];

fn git(cwd: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|e| format!("Failed to run git: {e}"))?;
    if out.status.success() {
        // Only the end: a leading space is meaningful in `git status --short`.
        Ok(String::from_utf8_lossy(&out.stdout).trim_end().to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

/// Undoes git's C-style quoting of paths with spaces or special characters
/// (`"a b.txt"`, `"tab\there"`, octal escapes for non-ASCII bytes when
/// core.quotePath is on).
fn unquote(path: &str) -> String {
    let Some(inner) = path.strip_prefix('"').and_then(|p| p.strip_suffix('"')) else {
        return path.to_string();
    };
    let mut bytes = Vec::with_capacity(inner.len());
    let mut chars = inner.bytes().peekable();
    while let Some(b) = chars.next() {
        if b != b'\\' {
            bytes.push(b);
            continue;
        }
        let escaped = chars
            .next()
            .unwrap_or_else(|| panic!("dangling backslash in git-quoted path {path:?}"));
        bytes.push(match escaped {
            b'a' => 0x07,
            b'b' => 0x08,
            b't' => b'\t',
            b'n' => b'\n',
            b'v' => 0x0b,
            b'f' => 0x0c,
            b'r' => b'\r',
            b'0'..=b'7' => {
                let mut value = u32::from(escaped - b'0');
                for _ in 0..2 {
                    match chars.peek() {
                        Some(d @ b'0'..=b'7') => {
                            value = value * 8 + u32::from(d - b'0');
                            chars.next();
                        }
                        _ => break,
                    }
                }
                u8::try_from(value)
                    .unwrap_or_else(|_| panic!("octal escape out of range in {path:?}"))
            }
            other => other,
        });
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

fn is_binary(path: &str) -> bool {
    path.rsplit_once('.').is_some_and(|(_, ext)| {
        BINARY_EXTENSIONS
            .iter()
            .any(|b| ext.eq_ignore_ascii_case(b))
    })
}

/// Files added, modified, renamed or untracked in the working tree and the
/// index, relative to `cwd`. Deleted and binary files are left out since
/// the point is to open the result in an editor.
pub fn modified_files_on_working_tree(cwd: &Path) -> Result<Vec<String>, String> {
    let status = git(cwd, &["status", "--short"])?;
    Ok(status
        .lines()
        .filter(|line| !line.is_empty())
        .filter_map(|line| {
            let (code, path) = line
                .split_at_checked(3)
                .unwrap_or_else(|| panic!("unexpected `git status --short` line: {line:?}"));
            if code.contains('D') {
                return None;
            }
            let path = if code.starts_with('R') || code.starts_with('C') {
                path.rsplit_once(" -> ")
                    .unwrap_or_else(|| panic!("rename line without ` -> `: {line:?}"))
                    .1
            } else {
                path
            };
            let path = unquote(path);
            (!is_binary(&path)).then_some(path)
        })
        .collect())
}

/// Files touched by `object`, relative to the repository root. For a merge
/// commit this is what the merged branch brought in, i.e. what a GitHub pull
/// request changed.
pub fn modified_files_on_commit(cwd: &Path, object: &str) -> Result<String, String> {
    let output = git(cwd, &["show", "--summary", "--format=%P", object])?;
    let parents: Vec<&str> = output
        .lines()
        .next()
        .unwrap_or_default()
        .split(' ')
        .collect();
    let files = match parents.as_slice() {
        [first, second] => git(
            cwd,
            &["diff", "--name-only", &format!("{first}...{second}")],
        ),
        _ => git(
            cwd,
            &["log", "-m", "-1", "--name-only", "--pretty=format:", object],
        ),
    };
    // `--pretty=format:` leaves a blank line before the file list.
    Ok(files?.trim_start().to_string())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let object = match args.as_slice() {
        [] => None,
        [flag] if flag == "-h" || flag == "--help" => {
            println!("{HELP}");
            return;
        }
        [flag] if flag == "-V" || flag == "--version" => {
            println!("{}", env!("CARGO_PKG_VERSION"));
            return;
        }
        [object] if !object.starts_with('-') => Some(object.as_str()),
        _ => {
            eprintln!("{HELP}");
            std::process::exit(2);
        }
    };

    let cwd = std::env::current_dir().unwrap_or_else(|e| {
        eprintln!("Failed to read the current directory: {e}");
        std::process::exit(1);
    });
    let result = match object {
        Some(object) => modified_files_on_commit(&cwd, object),
        None => modified_files_on_working_tree(&cwd).map(|files| files.join("\n")),
    };
    match result {
        Ok(files) if files.is_empty() => {}
        Ok(files) => println!("{files}"),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct Repo(PathBuf);

    impl Drop for Repo {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    impl Repo {
        /// A fresh repository with a.txt committed on `main`.
        fn new() -> Self {
            static SEQ: AtomicUsize = AtomicUsize::new(0);
            let dir = std::env::temp_dir().join(format!(
                "git_modified_files-test-{}-{}",
                std::process::id(),
                SEQ.fetch_add(1, Ordering::SeqCst)
            ));
            std::fs::create_dir_all(&dir).unwrap();
            let repo = Repo(dir);
            repo.touch("a.txt");
            repo.git(&["init", "--initial-branch=main"]);
            repo.git(&["config", "--local", "user.email", "hi@example.com"]);
            repo.git(&["config", "--local", "user.name", "yo"]);
            repo.git(&["config", "--local", "commit.gpgsign", "false"]);
            repo.git(&["add", "."]);
            repo.git(&["commit", "-m", "Init"]);
            repo
        }

        fn git(&self, args: &[&str]) -> String {
            git(&self.0, args).unwrap_or_else(|e| panic!("git {args:?} failed: {e}"))
        }

        fn touch(&self, path: &str) {
            std::fs::write(self.0.join(path), "").unwrap();
        }

        fn working_tree(&self) -> Vec<String> {
            modified_files_on_working_tree(&self.0).unwrap()
        }
    }

    #[test]
    fn no_output_without_modified_files() {
        assert!(Repo::new().working_tree().is_empty());
    }

    #[test]
    fn lists_staged_and_untracked_files() {
        let repo = Repo::new();
        repo.touch("b.txt");
        repo.git(&["add", "."]);
        repo.touch("c.txt");
        assert_eq!(repo.working_tree(), ["b.txt", "c.txt"]);
    }

    #[test]
    fn lists_files_modified_only_in_working_tree() {
        let repo = Repo::new();
        std::fs::write(repo.0.join("a.txt"), "changed").unwrap();
        assert_eq!(repo.working_tree(), ["a.txt"]);
    }

    #[test]
    fn skips_files_deleted_in_working_tree() {
        let repo = Repo::new();
        std::fs::remove_file(repo.0.join("a.txt")).unwrap();
        assert!(repo.working_tree().is_empty());
    }

    #[test]
    fn skips_files_deleted_in_index() {
        let repo = Repo::new();
        std::fs::remove_file(repo.0.join("a.txt")).unwrap();
        repo.git(&["add", "-u", "a.txt"]);
        assert!(repo.working_tree().is_empty());
    }

    #[test]
    fn skips_binary_files() {
        let repo = Repo::new();
        repo.touch("a.png");
        assert!(repo.working_tree().is_empty());
        repo.git(&["add", "a.png"]);
        assert!(repo.working_tree().is_empty());
    }

    #[test]
    fn lists_new_name_of_renamed_files() {
        let repo = Repo::new();
        std::fs::rename(repo.0.join("a.txt"), repo.0.join("b.txt")).unwrap();
        assert_eq!(repo.working_tree(), ["b.txt"]);
        repo.git(&["add", "-A"]);
        assert_eq!(repo.working_tree(), ["b.txt"]);
    }

    #[test]
    fn lists_paths_relative_to_current_directory() {
        let repo = Repo::new();
        std::fs::create_dir(repo.0.join("foo")).unwrap();
        repo.touch("foo/a.txt");
        repo.git(&["add", "."]);
        assert_eq!(
            modified_files_on_working_tree(&repo.0.join("foo")).unwrap(),
            ["a.txt"]
        );
    }

    #[test]
    fn unquotes_paths_with_spaces_and_non_ascii() {
        let repo = Repo::new();
        repo.git(&["config", "--local", "core.quotePath", "true"]);
        repo.touch("a b.txt");
        repo.touch("日本語.txt");
        assert_eq!(repo.working_tree(), ["a b.txt", "日本語.txt"]);
    }

    #[test]
    fn lists_files_of_a_commit() {
        let repo = Repo::new();
        let object = repo.git(&["log", "--pretty=format:%h", "-n", "1"]);
        assert_eq!(modified_files_on_commit(&repo.0, &object).unwrap(), "a.txt");
    }

    #[test]
    fn lists_files_brought_in_by_a_merge_commit() {
        let repo = Repo::new();
        for branch in ["b", "c"] {
            repo.git(&["checkout", "-b", &format!("{branch}-txt")]);
            repo.touch(&format!("{branch}.txt"));
            repo.git(&["add", "."]);
            repo.git(&["commit", "-m", &format!("{branch}.txt")]);
            repo.git(&["checkout", "main"]);
        }
        // --no-ff imitates a GitHub pull request merge
        repo.git(&["merge", "--no-ff", "c-txt"]);
        repo.git(&["merge", "--no-ff", "b-txt"]);

        let object = repo.git(&["log", "--pretty=format:%h", "-n", "1"]);
        assert_eq!(modified_files_on_commit(&repo.0, &object).unwrap(), "b.txt");
    }

    #[test]
    fn unquote_decodes_escapes() {
        assert_eq!(unquote("plain.txt"), "plain.txt");
        assert_eq!(unquote(r#""a b.txt""#), "a b.txt");
        assert_eq!(unquote(r#""tab\there\"q\\.txt""#), "tab\there\"q\\.txt");
        assert_eq!(unquote(r#""\346\227\245.txt""#), "日.txt");
    }
}
