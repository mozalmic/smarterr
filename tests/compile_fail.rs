use std::{path::Path, process::Command};

#[test]
fn unsupported_or_unsafe_conversions_are_compile_errors() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cases = [
        ("partial-question-mark", "From<ReadErrors>"),
        ("incomplete-match", "non-exhaustive patterns"),
        ("wrong-type", "mismatched types"),
        ("unknown-mapping", "mapping names an unknown source variant"),
        ("generic-error", "composition requires concrete error types"),
        ("non-tuple", "export/from requires an explicit tuple"),
        ("generic-source", "generic and associated source sets are not supported"),
        ("private-source", "enum `ReadErrors` is private"),
        ("private-module", "module `sets` is private"),
    ];
    for (bin, diagnostic) in cases {
        let output = Command::new(env!("CARGO"))
            .args(["check", "--manifest-path"])
            .arg(root.join("tests/cross-crate/Cargo.toml"))
            .args([
                "-p",
                "errorset-consumer",
                "--features",
                "compile-fail",
                "--bin",
                bin,
                "--color",
                "never",
            ])
            .output()
            .expect("could not run cargo for compile-fail fixture");
        let stderr = String::from_utf8(output.stderr).expect("cargo stderr is not UTF-8");
        assert!(!output.status.success(), "{bin} unexpectedly compiled");
        assert!(stderr.contains(diagnostic), "{bin}: expected {diagnostic:?}\n{stderr}");
    }
}
