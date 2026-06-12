//! Vwire 集成測試

use std::process::Command;

/// 測試 CLI 基本功能
#[test]
fn test_cli_help() {
    let output = Command::new("./target/release/vwire-filter")
        .arg("--help")
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--interface"));
}

/// 測試演示模式
#[test]
fn test_demo_mode() {
    let output = Command::new("./target/release/vwire-filter")
        .arg("--demo")
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
}
