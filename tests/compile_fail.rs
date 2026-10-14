use std::{path::Path, process::Command};

#[test]
fn unsupported_or_unsafe_conversions_are_compile_errors() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cases = [
        ("partial-question-mark", "couldn't convert the error to `PartialErrors`"),
        ("incomplete-match", "non-exhaustive patterns"),
        ("wrong-type", "mismatched types"),
        ("unknown-mapping", "mapping names an unknown source variant"),
        ("generic-error", "composition requires concrete error types"),
        ("non-tuple", "remove #[errorset] to return the atom directly, or use (Atom,)"),
        ("invalid-declarations", "errorset type declarations require an explicit tuple"),
        ("generic-source", "generic and associated source sets are not supported"),
        ("private-source", "macro import `ReadErrors` is private"),
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
        if bin == "non-tuple" {
            assert_eq!(stderr.matches(diagnostic).count(), 3, "{stderr}");
        }
        if bin == "partial-question-mark" {
            assert!(stderr.contains("couldn't convert the error to `NamedPartial`"), "{stderr}");
        }
        if bin == "invalid-declarations" {
            for diagnostic in [
                "errorset type declarations do not support generics or where clauses",
                "duplicate error atom at position 3; first declared at position 1",
                "first declaration of this error atom",
                "module/visibility options are not supported",
                "a set cannot convert from itself",
            ] {
                assert!(stderr.contains(diagnostic), "expected {diagnostic:?}\n{stderr}");
            }
            assert_eq!(
                stderr.matches("errorset type declarations do not support generics or where clauses").count(),
                3,
                "{stderr}",
            );
        }
    }
}
