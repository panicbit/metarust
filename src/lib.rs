#![allow(clippy::let_and_return)]

use std::{process::Command, fs};

use home::cargo_home;
use proc_macro::TokenStream;
use tempfile::tempdir;

#[proc_macro]
pub fn metarust(input: TokenStream) -> TokenStream {
    eval(&input.to_string())
        .parse()
        .unwrap()
}

fn eval(code: &str) -> String {
    let dir = tempdir().unwrap();
    let metarust_cache_dir = cargo_home()
        .expect("Failed to resolve cargo home")
        .join("metarust-cache");

    let output = Command::new("cargo")
        .args(&[
            "init",
            "--name=metarust_build",
            &dir.path().display().to_string(),
        ])
        .output()
        .expect("Failed to execute `cargo init`");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        panic!("Failed to initialize metarust scratch project:\n{stderr}");
    }

    let manifest_path = dir.path().join("Cargo.toml");

    fs::write(&manifest_path, r#"
        [package]
        name = "metarust_build"
        version = "0.1.0"
        edition = "2021"

        [dependencies]
        itertools = "0.10.3"
        proc-macro2 = "1.0.36"
        quote = "1.0.15"
    "#).expect("Failed to write Cargo.toml");

    let main_path = dir.path().join("src/main.rs");

    fs::write(main_path, format!(r#"
        #[allow(unused_imports)]
        use itertools::Itertools;
        #[allow(unused_imports)]
        use quote::{{format_ident, quote}};

        fn main() {{
            println!("{{}}", eval());
        }}

        fn eval() -> proc_macro2::TokenStream {{
            {code}
        }}
    "#)).expect("Failed to write main.rs");

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            &format!("--target-dir={}", metarust_cache_dir.display()),
            &format!("--manifest-path={}", manifest_path.display()),
            // TODO: add sccache for optimization
        ])
        .output()
        .expect("Failed to execute `cargo run`");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        panic!("Metarust:\n{stderr}");
    }

    let stdout = String::from_utf8(output.stdout).expect("Metarust output is not valid UTF-8!");

    stdout
}
