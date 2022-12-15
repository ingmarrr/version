use clap::Parser;
use serde::Deserialize;
use std::fmt::DebugStruct;
use std::io;
use std::io::Read;
use std::io::Write;

#[derive(Deserialize, Debug, Clone)]
struct Config {
    suffix: String,
    version: String,
}

impl Config {
    pub fn parse() -> Self {
        let mut file = std::fs::File::open(".vers").unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let mut iter = contents.split('\n');
        let version = iter.find(|s| s.starts_with("version"));
        let suffix = iter.find(|s| s.starts_with("suffix"));
        match (suffix, version) {
            (Some(s), Some(v)) => Self {
                suffix: s.replace("suffix = ", "").to_owned(),
                version: v.replace("version = ", "").to_owned(),
            },
            _ => Self {
                suffix: Suffix::Dev.to_string(),
                version: "1.0.0".to_owned(),
            },
        }
    }
}

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("suffix = {}\n", self.suffix))
    }
}

#[derive(Debug, Default)]
enum Suffix {
    #[default]
    Dev,
    Test,
    Rel,
    Alpha,
    Beta,
}

impl std::fmt::Display for Suffix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Suffix::Dev => f.write_str("dev"),
            Suffix::Test => f.write_str("test"),
            Suffix::Rel => f.write_str("rel"),
            Suffix::Alpha => f.write_str("alpha"),
            Suffix::Beta => f.write_str("beta"),
        }
    }
}

#[derive(clap::Parser)]
struct App {
    #[clap(subcommand)]
    cmd: Cmd,
}

#[derive(clap::Subcommand)]
enum Cmd {
    #[clap(name = "update")]
    Update,

    #[clap(name = "commit")]
    Commit(CommitOpts),

    #[clap(name = "push")]
    Push(PushOpts),

    #[clap(name = "tags")]
    Tags,
}

#[derive(clap::Args)]
struct CommitOpts {
    #[clap(long, short = 'm')]
    message: String,

    #[clap(long, short = 'o')]
    other: Option<String>,
}

#[derive(clap::Args)]
struct PushOpts {
    #[clap(long, short = 'o')]
    other: Option<String>,
}

#[derive(Debug)]
struct Version {
    major: u32,
    minor: u32,
    patch: u32,
}

impl Version {
    fn incr(&mut self) {
        self.patch += 1;
        if self.patch > 9 {
            self.patch = 0;
            self.minor += 1;
        }
        if self.minor > 9 {
            self.minor = 0;
            self.major += 1;
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "version = {}.{}.{}",
            self.major, self.minor, self.patch
        ))
    }
}

impl From<&str> for Version {
    fn from(s: &str) -> Self {
        let mut iter = s.split('.');
        let major = iter.next().unwrap().parse().unwrap();
        let minor = iter.next().unwrap().parse().unwrap();
        let patch = iter.next().unwrap().parse().unwrap();
        Self {
            major,
            minor,
            patch,
        }
    }
}

struct Rw(Config);

impl Rw {
    fn write(&self, version: Version, path: &str) {
        let mut file = std::fs::File::create(path).unwrap();
        let out = version.to_string() + "\n" + &self.0.to_string();
        file.write_all(out.as_bytes()).unwrap();
    }
}

fn commit(msg: &str, others: Vec<&str>) {
    let _others = match others.len() {
        0 => vec!["-a"],
        _ => others,
    };
    let cmd = std::process::Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(msg)
        .args(_others)
        .output()
        .unwrap();
    println!("status: {}", cmd.status);
    io::stdout().write_all(&cmd.stdout).unwrap();
    io::stderr().write_all(&cmd.stderr).unwrap();
}

fn push(others: Vec<&str>) {
    let _others = match others.len() {
        0 => vec!["origin", "main"],
        _ => others,
    };
    let cmd = std::process::Command::new("git")
        .arg("push")
        .args(_others)
        .output()
        .unwrap();
    println!("status: {}", cmd.status);
    io::stdout().write_all(&cmd.stdout).unwrap();
    io::stderr().write_all(&cmd.stderr).unwrap();
}

fn main() {
    let conf = Config::parse();
    let rw = Rw(conf.clone());
    let app = App::parse();

    match app.cmd {
        Cmd::Update => {
            let mut v = Version::from(conf.version.as_str());
            v.incr();
            rw.write(v, ".vers");
        }
        Cmd::Commit(op) => {
            let others = match op.other {
                Some(s) => s,
                None => "".to_owned(),
            };
            let other_args = others.split(' ').collect::<Vec<&str>>();
            commit(&op.message, other_args);
        }
        Cmd::Push(op) => {
            let others = match op.other {
                Some(s) => s,
                None => "".to_owned(),
            };
            let other_args = others.split(' ').collect::<Vec<&str>>();
            push(other_args);
        }
        Cmd::Tags => {
            let cmd = std::process::Command::new("git")
                .arg("push")
                .arg("--tag")
                .output()
                .unwrap();
            println!("status: {}", cmd.status);
            io::stdout().write_all(&cmd.stdout).unwrap();
            io::stderr().write_all(&cmd.stderr).unwrap();
        }
    }
}
