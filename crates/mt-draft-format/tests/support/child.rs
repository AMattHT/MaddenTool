//! Run one opt-in test of the current test binary in a child process with a chosen
//! environment. The variables are set on the child only, so parallel tests never race
//! on the parent's environment.

use std::ffi::OsStr;
use std::process::Command;

/// Every variable the opt-in tests read. The child starts with all of them removed.
const SETUP_VARS: [&str; 4] = [
    "MADDENTOOL_FIXTURES",
    "MADDENTOOL_LEGACY_MDG",
    "MADDENTOOL_REQUIRE_FIXTURES",
    "MADDENTOOL_NODE",
];

/// The child's outcome: whether the test passed, and its stdout and stderr together.
pub struct ChildRun {
    pub passed: bool,
    pub output: String,
}

pub fn run_test_in_child(test: &str, env: &[(&str, &OsStr)]) -> ChildRun {
    let exe = std::env::current_exe().expect("current test binary");
    let mut cmd = Command::new(exe);
    cmd.args([test, "--exact", "--nocapture", "--test-threads=1"]);
    for var in SETUP_VARS {
        cmd.env_remove(var);
    }
    for (k, v) in env {
        cmd.env(k, v);
    }
    let out = cmd
        .output()
        .expect("run the test binary in a child process");
    let output = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        output.contains("running 1 test"),
        "the child did not run exactly {test}: {output}"
    );
    ChildRun {
        passed: out.status.success(),
        output,
    }
}
