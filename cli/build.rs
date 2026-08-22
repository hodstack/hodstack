use std::env;
use std::error::Error;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let out = PathBuf::from(env::var("OUT_DIR")?);

    let mut code = String::from("static SKILLS: &[Built] = &[\n");

    watch(&packaged(&manifest));
    watch(&repository(&manifest));

    if let Some(tree) = tree(&manifest) {
        for skill in read(&tree)? {
            if !skill.is_dir() {
                continue;
            }

            if !skill.join("SKILL.md").is_file() {
                return Err(format!("`{}` holds no SKILL.md", skill.display()).into());
            }

            let name = name(&skill);
            writeln!(code, "    Built {{ name: {name:?}, files: &[")?;

            for file in walk(&skill)? {
                let relative = relative(&skill, &file);
                let path = file.display().to_string();

                watch(&file);
                writeln!(code, "        ({relative:?}, include_str!({path:?})),")?;
            }

            code.push_str("    ] },\n");
        }
    }

    code.push_str("];\n");

    fs::write(out.join("skills.rs"), code)?;

    Ok(())
}

fn tree(manifest: &Path) -> Option<PathBuf> {
    [packaged(manifest), repository(manifest)]
        .into_iter()
        .find(|tree| tree.is_dir())
}

fn packaged(manifest: &Path) -> PathBuf {
    manifest.join("skills")
}

fn repository(manifest: &Path) -> PathBuf {
    manifest.join("../skills/skills")
}

fn read(dir: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut paths = Vec::new();

    for entry in fs::read_dir(dir)? {
        let path = entry?.path();

        if name(&path).starts_with('.') {
            continue;
        }

        paths.push(path);
    }

    paths.sort();

    Ok(paths)
}

fn walk(dir: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut files = Vec::new();

    for path in read(dir)? {
        if path.is_dir() {
            files.extend(walk(&path)?);
        } else {
            files.push(path);
        }
    }

    Ok(files)
}

fn name(path: &Path) -> String {
    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

fn relative(skill: &Path, file: &Path) -> String {
    file.strip_prefix(skill)
        .unwrap_or(file)
        .to_string_lossy()
        .replace('\\', "/")
}

fn watch(path: &Path) {
    println!("cargo::rerun-if-changed={}", path.display());
}
