use std::env;
use std::fs;
use std::io::{self, IsTerminal as _, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::Duration;

use anstyle::Style;
use anyhow::{Context as _, Result, anyhow, bail};
use sha2::{Digest as _, Sha256};

const BOLD: Style = Style::new().bold();

const DIM: Style = Style::new().dimmed();

const RELEASE: &str = "https://github.com/hodstack/hodstack/releases/latest/download";

const DAY: Duration = Duration::from_secs(60 * 60 * 24);

#[derive(Debug, PartialEq, Eq)]
struct Build {
    version: String,
    commit: String,
}

impl Build {
    fn read(text: &str) -> Result<Self> {
        let mut fields = text.split_whitespace();

        let version = fields
            .next()
            .ok_or_else(|| anyhow!("version.txt names no version"))?;
        let commit = fields
            .next()
            .ok_or_else(|| anyhow!("version.txt names no commit"))?;

        Ok(Self {
            version: version.to_owned(),
            commit: commit.to_owned(),
        })
    }

    fn write(&self) -> String {
        format!("{} {}\n", self.version, self.commit)
    }

    fn label(&self) -> String {
        format!("{} ({})", self.version, short(&self.commit))
    }
}

pub fn update(check: bool, out: &mut impl Write) -> Result<ExitCode> {
    let path = env::current_exe().context("cannot read the path of this program")?;
    let binary = fs::canonicalize(&path).unwrap_or_else(|_| path.clone());

    let newest = newest()?;
    remember(&newest);

    writeln!(out)?;

    if running() == Some(newest.commit.as_str()) {
        writeln!(out, "  {BOLD}Current{BOLD:#}  {}", newest.label())?;
        writeln!(out, "           {DIM}{}{DIM:#}", path.display())?;
        writeln!(out)?;

        return Ok(ExitCode::SUCCESS);
    }

    if check {
        writeln!(out, "  {BOLD}Newest{BOLD:#}   {}", newest.label())?;
        writeln!(out, "           Run `hod update` to install it.")?;
        writeln!(out)?;

        return Ok(ExitCode::SUCCESS);
    }

    install(&binary, &newest)?;

    writeln!(
        out,
        "  {BOLD}Updated{BOLD:#}  {} → {}",
        crate::version(),
        newest.label()
    )?;
    writeln!(out, "           {DIM}{}{DIM:#}", path.display())?;
    writeln!(out)?;

    Ok(ExitCode::SUCCESS)
}

pub fn notice(err: &mut impl Write) {
    if env::var_os("HOD_NO_UPDATE_CHECK").is_some() {
        return;
    }

    if env::var_os("CI").is_some() {
        return;
    }

    if !io::stderr().is_terminal() {
        return;
    }

    let Some(running) = running() else {
        return;
    };

    let Some(path) = cache() else {
        return;
    };

    let known = fs::read_to_string(&path).ok();

    if let Some(build) = known.as_deref().and_then(|text| Build::read(text).ok()) {
        if build.commit != running {
            let _ = writeln!(
                err,
                "  {DIM}A newer hod is available. Run `hod update`.{DIM:#}\n"
            );
        }
    }

    if fresh(&path) {
        return;
    }

    if !touch(&path, known.unwrap_or_default().as_str()) {
        return;
    }

    let _ = spawn_check();
}

fn running() -> Option<&'static str> {
    option_env!("HOD_COMMIT")
}

fn short(commit: &str) -> &str {
    commit.get(..7).unwrap_or(commit)
}

fn address() -> String {
    env::var("HOD_RELEASE_URL").unwrap_or_else(|_| RELEASE.to_owned())
}

fn asset() -> Result<&'static str> {
    let target = match (env::consts::OS, env::consts::ARCH) {
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("linux", "aarch64") => "aarch64-unknown-linux-musl",
        ("linux", "x86_64") => "x86_64-unknown-linux-musl",
        ("windows", "x86_64") => "x86_64-pc-windows-msvc",
        (os, arch) => bail!("no build for {os} {arch}"),
    };

    Ok(target)
}

fn newest() -> Result<Build> {
    let work = work(&env::temp_dir())?;
    let path = work.join("version.txt");

    let result = fetch(&format!("{}/version.txt", address()), &path)
        .and_then(|()| fs::read_to_string(&path).context("cannot read version.txt"))
        .and_then(|text| Build::read(&text));

    let _ = fs::remove_dir_all(&work);

    result
}

fn install(binary: &Path, newest: &Build) -> Result<()> {
    let parent = binary
        .parent()
        .ok_or_else(|| anyhow!("cannot read the directory of `{}`", binary.display()))?;

    let name = binary
        .file_name()
        .ok_or_else(|| anyhow!("cannot read the name of `{}`", binary.display()))?
        .to_owned();

    let file = format!("hod-{}.tar.gz", asset()?);
    let work = work(parent)?;

    let result = download(&work, &file).and_then(|()| {
        let new = work.join(if cfg!(windows) { "hod.exe" } else { "hod" });

        permit(&new)?;
        swap(&new, binary, parent, name.as_ref())
    });

    let _ = fs::remove_dir_all(&work);

    result.with_context(|| format!("cannot install {}", newest.label()))
}

fn download(work: &Path, file: &str) -> Result<()> {
    let address = address();
    let archive = work.join(file);
    let checksums = work.join("checksums.txt");

    fetch(&format!("{address}/{file}"), &archive)?;
    fetch(&format!("{address}/checksums.txt"), &checksums)?;

    let checksums = fs::read_to_string(&checksums).context("cannot read checksums.txt")?;
    let wanted = wanted_sum(&checksums, file)?;
    let found = sum(&archive)?;

    if wanted != found {
        bail!("the checksum of {file} does not agree with checksums.txt");
    }

    extract(&archive, work)
}

fn wanted_sum<'a>(checksums: &'a str, file: &str) -> Result<&'a str> {
    checksums
        .lines()
        .find(|line| line.ends_with(&format!(" {file}")))
        .and_then(|line| line.split_whitespace().next())
        .ok_or_else(|| anyhow!("checksums.txt names no {file}"))
}

fn sum(path: &Path) -> Result<String> {
    let bytes = fs::read(path).with_context(|| format!("cannot read `{}`", path.display()))?;

    Ok(format!("{:x}", Sha256::digest(&bytes)))
}

fn extract(archive: &Path, work: &Path) -> Result<()> {
    let mut tar = Command::new("tar");
    tar.arg("-xzf").arg(archive).arg("-C").arg(work);

    match outcome(&mut tar) {
        Some(result) => {
            result.with_context(|| format!("cannot open `{}`", archive.display()))?;
            Ok(())
        }
        None => bail!("this computer has no tar"),
    }
}

fn fetch(address: &str, path: &Path) -> Result<()> {
    let mut curl = Command::new("curl");
    curl.args([
        "--fail",
        "--silent",
        "--show-error",
        "--location",
        "--max-time",
        "120",
    ])
    .arg(address)
    .arg("--output")
    .arg(path);

    if let Some(result) = outcome(&mut curl) {
        return result.with_context(|| format!("cannot download {address}"));
    }

    let mut wget = Command::new("wget");
    wget.args(["--quiet", "--timeout", "120"])
        .arg(address)
        .arg("--output-document")
        .arg(path);

    if let Some(result) = outcome(&mut wget) {
        return result.with_context(|| format!("cannot download {address}"));
    }

    bail!("this computer has no curl and no wget")
}

fn outcome(command: &mut Command) -> Option<Result<()>> {
    let output = match command.output() {
        Ok(output) => output,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return None,
        Err(error) => return Some(Err(error.into())),
    };

    if output.status.success() {
        return Some(Ok(()));
    }

    let message = String::from_utf8_lossy(&output.stderr).trim().to_owned();

    Some(Err(if message.is_empty() {
        anyhow!("the program reports {}", output.status)
    } else {
        anyhow!(message)
    }))
}

fn work(parent: &Path) -> Result<PathBuf> {
    let work = parent.join(format!(".hod-update-{}", std::process::id()));

    let _ = fs::remove_dir_all(&work);
    fs::create_dir_all(&work).with_context(|| format!("cannot write in `{}`", parent.display()))?;

    Ok(work)
}

fn permit(new: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;

        fs::set_permissions(new, fs::Permissions::from_mode(0o755))
            .with_context(|| format!("cannot give `{}` its mode", new.display()))?;
    }

    #[cfg(not(unix))]
    {
        let _ = new;
    }

    Ok(())
}

fn swap(new: &Path, binary: &Path, parent: &Path, name: &Path) -> Result<()> {
    let old = parent.join(format!(".{}.old", name.display()));

    let _ = fs::remove_file(&old);
    fs::rename(binary, &old).with_context(|| format!("cannot move `{}`", binary.display()))?;

    match fs::rename(new, binary) {
        Ok(()) => {
            let _ = fs::remove_file(&old);
            Ok(())
        }
        Err(error) => {
            let _ = fs::rename(&old, binary);
            Err(error).with_context(|| format!("cannot write `{}`", binary.display()))
        }
    }
}

fn cache() -> Option<PathBuf> {
    let base = if cfg!(windows) {
        env::var_os("LOCALAPPDATA").map(PathBuf::from)
    } else {
        env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
    };

    Some(base?.join("hod").join("update"))
}

fn remember(newest: &Build) {
    let Some(path) = cache() else {
        return;
    };

    touch(&path, &newest.write());
}

fn touch(path: &Path, text: &str) -> bool {
    let Some(parent) = path.parent() else {
        return false;
    };

    if fs::create_dir_all(parent).is_err() {
        return false;
    }

    fs::write(path, text).is_ok()
}

fn fresh(path: &Path) -> bool {
    let Ok(data) = fs::metadata(path) else {
        return false;
    };

    let Ok(time) = data.modified() else {
        return false;
    };

    match time.elapsed() {
        Ok(age) => age < DAY,
        Err(_) => true,
    }
}

fn spawn_check() -> Result<()> {
    let program = env::current_exe()?;

    Command::new(program)
        .args(["update", "--check"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_build_reads_a_version_and_a_commit() {
        let build = Build::read("0.0.1 3f2b1c9d8e7f6a5b4c3d2e1f\n").unwrap();

        assert_eq!(build.version, "0.0.1");
        assert_eq!(build.commit, "3f2b1c9d8e7f6a5b4c3d2e1f");
        assert_eq!(build.label(), "0.0.1 (3f2b1c9)");
        assert_eq!(build.write(), "0.0.1 3f2b1c9d8e7f6a5b4c3d2e1f\n");
    }

    #[test]
    fn a_build_without_a_commit_is_a_fault() {
        assert!(Build::read("").is_err());
        assert!(Build::read("0.0.1").is_err());
    }

    #[test]
    fn a_short_commit_keeps_its_length() {
        assert_eq!(short("abc"), "abc");
        assert_eq!(short("3f2b1c9d8e7f"), "3f2b1c9");
    }

    #[test]
    fn the_checksums_give_the_sum_of_one_file() {
        let checksums =
            "aaa  hod-x86_64-apple-darwin.tar.gz\nbbb  hod-aarch64-apple-darwin.tar.gz\n";

        assert_eq!(
            wanted_sum(checksums, "hod-aarch64-apple-darwin.tar.gz").unwrap(),
            "bbb"
        );
        assert!(wanted_sum(checksums, "hod-x86_64-pc-windows-msvc.tar.gz").is_err());
    }

    #[test]
    fn a_file_that_does_not_exist_is_not_fresh() {
        assert!(!fresh(Path::new("/hod/no/such/file")));
    }
}
