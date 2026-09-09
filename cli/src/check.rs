use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::ExitCode;

use anstyle::Style;
use anyhow::{Context as _, Result};

use crate::project::{self, Project};

const DIM: Style = Style::new().dimmed();

const IGNORED: [&str; 4] = [".git", "target", "node_modules", "vendor"];

const SOURCE: [&str; 11] = [
    ".rs", ".toml", ".md", ".yml", ".yaml", ".json", ".sh", ".ps1", ".js", ".lock", ".txt",
];

#[derive(Debug, Default)]
struct Tree {
    files: BTreeSet<String>,
    dirs: BTreeSet<String>,
}

#[derive(Debug, PartialEq, Eq)]
enum Reference {
    Path(String),
    Name(String, String),
}

#[derive(Debug)]
struct Fault {
    at: String,
    says: String,
}

pub fn check(project: &Project, out: &mut impl Write) -> Result<ExitCode> {
    let tree = read(project.root())?;
    let documents = documents(&tree);
    let mut faults = Vec::new();
    let mut paths = 0;
    let mut names = 0;

    for document in &documents {
        let text = fs::read_to_string(project.path(document))
            .with_context(|| format!("cannot read `{document}`"))?;
        let base = base(document);

        for (line, reference) in references(&text) {
            let at = || format!("{document}:{line}");

            match reference {
                Reference::Path(path) => {
                    if !rooted(&path, base, &tree) {
                        continue;
                    }

                    paths += 1;

                    if resolve(&path, base, &tree).is_none() {
                        faults.push(Fault {
                            at: at(),
                            says: format!("`{path}` names no file of this project"),
                        });
                    }
                }
                Reference::Name(name, path) => {
                    if !rooted(&path, base, &tree) {
                        continue;
                    }

                    let Some(file) = resolve(&path, base, &tree) else {
                        continue;
                    };

                    let source = fs::read_to_string(project.path(&file))
                        .with_context(|| format!("cannot read `{file}`"))?;

                    names += 1;

                    if !holds(&source, &name) {
                        faults.push(Fault {
                            at: at(),
                            says: format!("`{name}` is absent from `{path}`"),
                        });
                    }
                }
            }
        }
    }

    report(out, documents.len(), paths, names, &faults)
}

fn report(
    out: &mut impl Write,
    documents: usize,
    paths: usize,
    names: usize,
    faults: &[Fault],
) -> Result<ExitCode> {
    writeln!(out)?;

    if faults.is_empty() {
        writeln!(
            out,
            "  {}, {}, {}. Each one exists.",
            count(documents, "file"),
            count(paths, "path"),
            count(names, "name")
        )?;
        writeln!(out)?;

        return Ok(ExitCode::SUCCESS);
    }

    let width = faults.iter().map(|fault| fault.at.len()).max().unwrap_or(0);

    for fault in faults {
        writeln!(
            out,
            "  {DIM}{:width$}{DIM:#}  {}",
            fault.at,
            fault.says,
            width = width
        )?;
    }

    writeln!(out)?;
    writeln!(out, "  {}.", count(faults.len(), "fault"))?;
    writeln!(out)?;

    Ok(ExitCode::FAILURE)
}

fn count(number: usize, one: &str) -> String {
    if number == 1 {
        return format!("{number} {one}");
    }

    format!("{number} {one}s")
}

fn read(root: &Path) -> Result<Tree> {
    let mut tree = Tree::default();

    walk(root, root, &mut tree)?;

    Ok(tree)
}

fn walk(root: &Path, dir: &Path, tree: &mut Tree) -> Result<()> {
    let entries = fs::read_dir(dir).with_context(|| format!("cannot read `{}`", dir.display()))?;

    for entry in entries {
        let path = entry
            .with_context(|| format!("cannot read `{}`", dir.display()))?
            .path();

        if !path.is_dir() {
            tree.files.insert(project::relative(root, &path));
            continue;
        }

        if IGNORED.contains(&project::name(&path).as_str()) {
            continue;
        }

        tree.dirs.insert(project::relative(root, &path));
        walk(root, &path, tree)?;
    }

    Ok(())
}

fn documents(tree: &Tree) -> Vec<String> {
    let mut documents = BTreeSet::new();

    for file in &tree.files {
        if Path::new(file)
            .extension()
            .is_none_or(|extension| extension != "md")
        {
            continue;
        }

        let root = !file.contains('/');
        let rules = file.starts_with(".hod/");
        let agents = project::name(Path::new(file)) == project::AGENTS && !hidden(file);

        if root || rules || agents {
            documents.insert(file.clone());
        }
    }

    documents.into_iter().collect()
}

fn hidden(file: &str) -> bool {
    file.split('/').any(|part| part.starts_with('.'))
}

fn base(document: &str) -> &str {
    document.rsplit_once('/').map_or("", |(base, _)| base)
}

fn references(text: &str) -> Vec<(usize, Reference)> {
    let mut found = Vec::new();
    let mut fenced = false;

    for (at, line) in text.lines().enumerate() {
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
            continue;
        }

        if fenced {
            continue;
        }

        for reference in row(line) {
            found.push((at + 1, reference));
        }
    }

    found
}

fn row(line: &str) -> Vec<Reference> {
    let parts: Vec<&str> = line.split('`').collect();
    let last = parts.len().saturating_sub(1);
    let mut found = Vec::new();
    let mut at = 1;

    while at < last {
        let token = parts[at];

        if let Some(path) = path(token) {
            found.push(Reference::Path(path));
        } else if identifier(token) && parts[at + 1] == " in " && at + 2 < last {
            if let Some(path) = path(parts[at + 2]) {
                found.push(Reference::Name(token.to_owned(), path));
            }
        }

        at += 2;
    }

    found
}

fn path(token: &str) -> Option<String> {
    if token.contains([' ', '*', '<', '>']) || token.starts_with('@') || token.contains("://") {
        return None;
    }

    if !token.trim_matches('/').contains('/') {
        return None;
    }

    if !SOURCE.iter().any(|extension| token.ends_with(extension)) {
        return None;
    }

    Some(token.to_owned())
}

fn identifier(token: &str) -> bool {
    let mut letters = token.chars();

    letters
        .next()
        .is_some_and(|first| first.is_alphabetic() || first == '_')
        && letters.all(|letter| letter.is_alphanumeric() || letter == '_')
}

fn rooted(token: &str, base: &str, tree: &Tree) -> bool {
    let head = token
        .trim_start_matches('/')
        .split('/')
        .next()
        .unwrap_or_default();

    if tree.dirs.contains(head) {
        return true;
    }

    !base.is_empty() && tree.dirs.contains(&format!("{base}/{head}"))
}

fn resolve(token: &str, base: &str, tree: &Tree) -> Option<String> {
    let path = token.trim_start_matches('/');

    if !base.is_empty() {
        let near = format!("{base}/{path}");

        if tree.files.contains(&near) {
            return Some(near);
        }
    }

    if tree.files.contains(path) {
        return Some(path.to_owned());
    }

    let tail = format!("/{path}");

    tree.files
        .iter()
        .find(|file| file.ends_with(&tail))
        .cloned()
}

fn holds(text: &str, name: &str) -> bool {
    text.match_indices(name).any(|(at, _)| {
        let before = text[..at].chars().next_back();
        let after = text[at + name.len()..].chars().next();

        !before.is_some_and(letter) && !after.is_some_and(letter)
    })
}

fn letter(one: char) -> bool {
    one.is_alphanumeric() || one == '_'
}

#[cfg(test)]
mod tests {
    use super::*;

    fn faults(dir: &Path) -> (ExitCode, String) {
        let mut out = Vec::new();
        let code = check(&Project::new(dir), &mut out).unwrap();

        (code, String::from_utf8(out).unwrap())
    }

    #[test]
    fn a_path_in_code_font_is_a_reference() {
        assert_eq!(
            row("The file `src/cli.rs` holds the text."),
            vec![Reference::Path("src/cli.rs".to_owned())]
        );
    }

    #[test]
    fn a_name_before_a_path_is_a_reference_and_the_path_is_one_too() {
        assert_eq!(
            row("The table `AGENTS` in `cli/src/agent.rs` names five."),
            vec![
                Reference::Name("AGENTS".to_owned(), "cli/src/agent.rs".to_owned()),
                Reference::Path("cli/src/agent.rs".to_owned()),
            ]
        );
    }

    #[test]
    fn a_command_a_supplier_and_a_pattern_are_no_path() {
        for line in [
            "Run `cargo make test` before you commit.",
            "Give `dtolnay/rust-toolchain` the SHA of the tag `v1`.",
            "The job publishes `@hodstack/cli` and `@hodstack/hod`.",
            "The command `hod pr-review` starts `skills/skills/<name>/SKILL.md`.",
            "`Cargo.toml` names `/skills/**` in the key `include`.",
            "The address is `https://github.com/hodstack/hodstack/install.sh`.",
        ] {
            assert!(row(line).is_empty(), "`{line}` gives a reference");
        }
    }

    #[test]
    fn a_fenced_block_gives_no_reference() {
        let text = "The file `src/cli.rs`.\n```text\n`src/gone.rs`\n```\n`src/help.rs`\n";

        assert_eq!(
            references(text),
            vec![
                (1, Reference::Path("src/cli.rs".to_owned())),
                (5, Reference::Path("src/help.rs".to_owned())),
            ]
        );
    }

    #[test]
    fn a_name_inside_a_longer_word_is_not_that_name() {
        assert!(holds("pub const CLIENTS: [&str; 2] = [];", "CLIENTS"));
        assert!(!holds("pub const CLIENTSX: [&str; 2] = [];", "CLIENTS"));
        assert!(!holds("pub const XCLIENTS: [&str; 2] = [];", "CLIENTS"));
    }

    #[test]
    fn a_project_without_a_fault_reports_each_count() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("cli/src")).unwrap();
        fs::write(
            dir.path().join("cli/src/agent.rs"),
            "pub const AGENTS: u8 = 0;",
        )
        .unwrap();
        fs::write(
            dir.path().join("AGENTS.md"),
            "The table `AGENTS` in `cli/src/agent.rs` names five.\n",
        )
        .unwrap();

        let (code, out) = faults(dir.path());

        assert_eq!(code, ExitCode::SUCCESS);
        assert_eq!(out, "\n  1 file, 1 path, 1 name. Each one exists.\n\n");
    }

    #[test]
    fn a_path_that_no_file_carries_is_a_fault() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("cli/src")).unwrap();
        fs::write(
            dir.path().join("cli/src/agent.rs"),
            "pub const AGENTS: u8 = 0;",
        )
        .unwrap();
        fs::write(
            dir.path().join("AGENTS.md"),
            "Read `cli/src/gone.rs` and `cli/src/agent.rs`.\n",
        )
        .unwrap();

        let (code, out) = faults(dir.path());

        assert_eq!(code, ExitCode::FAILURE);
        assert!(
            out.contains("AGENTS.md:1") && out.contains("`cli/src/gone.rs` names no file"),
            "{out}"
        );
        assert!(out.contains("1 fault."), "{out}");
    }

    #[test]
    fn a_name_that_its_file_does_not_carry_is_a_fault() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("cli/src")).unwrap();
        fs::write(
            dir.path().join("cli/src/agent.rs"),
            "pub const AGENTS: u8 = 0;",
        )
        .unwrap();
        fs::write(
            dir.path().join("AGENTS.md"),
            "The table `CLIENTS` in `cli/src/agent.rs` names five.\n",
        )
        .unwrap();

        let (code, out) = faults(dir.path());

        assert_eq!(code, ExitCode::FAILURE);
        assert!(
            out.contains("AGENTS.md:1")
                && out.contains("`CLIENTS` is absent from `cli/src/agent.rs`"),
            "{out}"
        );
    }

    #[test]
    fn a_path_reads_against_the_directory_of_its_file_and_against_the_root() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("cli/src")).unwrap();
        fs::create_dir_all(dir.path().join(".github/workflows")).unwrap();
        fs::write(dir.path().join("cli/src/cli.rs"), "").unwrap();
        fs::write(dir.path().join(".github/workflows/ci.yml"), "").unwrap();
        fs::write(
            dir.path().join("cli/AGENTS.md"),
            "Read `src/cli.rs` and `.github/workflows/ci.yml`.\n",
        )
        .unwrap();

        let (code, out) = faults(dir.path());

        assert_eq!(code, ExitCode::SUCCESS, "{out}");
        assert!(out.contains("2 paths"), "{out}");
    }

    #[test]
    fn a_path_with_a_first_directory_that_is_absent_is_no_reference_of_this_project() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("AGENTS.md"),
            "The user writes `.hod/PROJECT.md`, and the address is `releases/latest/install.sh`.\n",
        )
        .unwrap();

        let (code, out) = faults(dir.path());

        assert_eq!(code, ExitCode::SUCCESS);
        assert!(out.contains("0 paths"), "{out}");
    }

    #[test]
    fn the_documents_are_each_agents_file_each_file_of_hod_and_each_file_at_the_root() {
        let mut tree = Tree::default();
        for file in [
            "AGENTS.md",
            "CONTEXT.md",
            "cli/AGENTS.md",
            ".hod/PROJECT.md",
            ".hod/rules/one.md",
            ".agents/skills/other/AGENTS.md",
            "cli/README.md",
            "cli/src/lib.rs",
        ] {
            tree.files.insert(file.to_owned());
        }

        assert_eq!(
            documents(&tree),
            vec![
                ".hod/PROJECT.md",
                ".hod/rules/one.md",
                "AGENTS.md",
                "CONTEXT.md",
                "cli/AGENTS.md",
            ]
        );
    }
}
