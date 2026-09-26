use std::{
    env::{
        self,
        consts::{ARCH, OS},
    },
    process::Command,
};

const BASE_URL: &str =
    "https://github.com/tailwindlabs/tailwindcss/releases/download/v4.3.3/tailwindcss-";
const SCRIPT: &str = r#"
set -euo pipefail

if [ ! -e tailwindcss ]; then
  curl -Lo tailwindcss $0
  chmod +x tailwindcss
fi

echo '@import "tailwindcss";' | ./tailwindcss -i - -o $OUT_DIR/styles.css --cwd $CARGO_MANIFEST_DIR/templates --silent
"#;

fn main() {
    let suffix = match (ARCH, OS) {
        ("x86_64", "linux") => "linux-x64-musl",
        ("x86_64", "macos") => "macos-x64",
        ("aarch64", "linux") => "linux-arm64-musl",
        ("aarch64", "macos") => "macos-arm64",
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
