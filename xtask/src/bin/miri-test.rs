use std::process::{Command, Stdio};

///
/// ```sh
/// MIRIFLAGS=-Zmiri-disable-isolation cargo miri test --features nightly test_gap --lib
/// ```
fn main() {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let cwd = std::env::current_dir().unwrap();
    let mut cmd = Command::new(cargo);
    cmd.current_dir(cwd);
    cmd.envs(std::env::vars());
    cmd.env("MIRIFLAGS", "-Zmiri-disable-isolation");
    cmd.args(["miri", "test", "--features", "nightly"]);
    cmd.args(std::env::args().skip(1));
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());
    let mut child = cmd.spawn().unwrap();
    _ = child.wait();
}
