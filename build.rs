use std::{
    env::{self},
    process::Command,
};

const BASE_URL: &str =
    "https://github.com/tailwindlabs/tailwindcss/releases/download/v4.3.3/tailwindcss-";
const SCRIPT: &str = r#"
set -euo pipefail

command -v tailwindcss &> /dev/null && in_path=0 || in_path=1

command=./tailwindcss
if [ $in_path -eq 0 ]; then command=tailwindcss; fi

if [ ! -e tailwindcss ] && [ $in_path -eq 1 ]; then
  curl -Lo tailwindcss $0
  chmod +x tailwindcss
fi

echo '@import "tailwindcss";' | $command -i - -o $OUT_DIR/styles.css --cwd $CARGO_MANIFEST_DIR/templates --silent
"#;

fn main() {
    let suffix = match env::var("HOST").unwrap().as_ref() {
        "x86_64-unknown-linux-gnu" => "linux-x64",
        "x86_64-unknown-linux-musl" => "linux-x64-musl",
        "aarch64-unknown-linux-gnu" => "linux-arm64",
        "aarch64-unknown-linux-musl" => "linux-arm64-musl",
        "aarch64-apple-darwin" => "macos-arm64",
        _ => panic!("unsupported environment"),
    };
    assert!(
        Command::new("bash")
            .arg("-c")
            .arg(SCRIPT)
            .arg(format!("{BASE_URL}{suffix}"))
            .current_dir(env::var("OUT_DIR").unwrap())
            .status()
            .unwrap()
            .success()
    );
}
