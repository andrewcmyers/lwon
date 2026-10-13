use std::path::Path;
use std::process::Command;

/// Runs the language-independent tests in the repository's tests/ directory.
#[test]
fn shared_tests() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let status = Command::new("perl")
        .arg("bin/run-tests")
        .arg(env!("CARGO_BIN_EXE_lwon-dump"))
        .current_dir(root)
        .status()
        .expect("cannot run bin/run-tests");
    assert!(status.success());
}
