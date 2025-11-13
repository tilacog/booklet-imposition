//! Subprocess utilities for orchestrating external PDF tools

use crate::error::{PdfError, Result};
use std::process::{Command, Output};

/// Version of pdfbook2 detected
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pdfbook2Version {
    /// Modern version with --outfile support
    Modern,
    /// Legacy version without --outfile
    Legacy,
}

/// Check if a command exists in PATH
///
/// # Arguments
///
/// * `cmd` - Command name to check
///
/// # Errors
///
/// Returns an error if:
/// - `which` command fails to execute
/// - Command is not found in PATH
pub fn check_dependency(cmd: &str) -> Result<()> {
    // Try to find the command using 'which'
    let output = Command::new("which")
        .arg(cmd)
        .output()
        .map_err(|e| PdfError::SubprocessFailed(format!("Failed to run 'which': {e}")))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(PdfError::MissingDependency(cmd.to_string()))
    }
}

/// Run a command and return its output
///
/// # Arguments
///
/// * `cmd` - Command to execute
///
/// # Errors
///
/// Returns an error if:
/// - Command execution fails
pub fn run_command(cmd: &mut Command) -> Result<Output> {
    let program = cmd
        .get_program()
        .to_str()
        .unwrap_or("unknown")
        .to_string();

    let output = cmd.output().map_err(|e| {
        PdfError::SubprocessFailed(format!("Failed to execute {program}: {e}"))
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(PdfError::SubprocessFailed(format!(
            "{} failed with status {}: {}",
            program,
            output.status,
            stderr.trim()
        )));
    }

    Ok(output)
}

/// Detect which version of pdfbook2 is installed
///
/// Modern versions support --outfile flag, legacy versions don't
///
/// # Errors
///
/// Returns an error if:
/// - `pdfbook2 --help` command fails to execute
pub fn detect_pdfbook2_version() -> Result<Pdfbook2Version> {
    let output = Command::new("pdfbook2")
        .arg("--help")
        .output()
        .map_err(|e| PdfError::SubprocessFailed(format!("Failed to run pdfbook2: {e}")))?;

    let help_text = String::from_utf8_lossy(&output.stdout);
    let help_err = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{help_text}{help_err}");

    if combined.contains("--outfile") {
        Ok(Pdfbook2Version::Modern)
    } else {
        Ok(Pdfbook2Version::Legacy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires 'which' command to be available
    fn test_check_dependency_exists() {
        // 'sh' should exist on all Unix systems
        assert!(check_dependency("sh").is_ok());
    }

    #[test]
    #[ignore] // Requires 'which' command to be available
    fn test_check_dependency_missing() {
        // This command should not exist
        let result = check_dependency("this-command-definitely-does-not-exist-12345");
        assert!(result.is_err());

        if let Err(PdfError::MissingDependency(cmd)) = result {
            assert_eq!(cmd, "this-command-definitely-does-not-exist-12345");
        } else {
            panic!("Expected MissingDependency error");
        }
    }

    #[test]
    #[ignore] // Requires system command
    fn test_run_command_success() {
        let mut cmd = Command::new("echo");
        cmd.arg("test");

        let result = run_command(&mut cmd);
        assert!(result.is_ok());

        let output = result.unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout.trim(), "test");
    }

    #[test]
    #[ignore] // Requires system command
    fn test_run_command_failure() {
        let mut cmd = Command::new("false"); // Command that always fails

        let result = run_command(&mut cmd);
        assert!(result.is_err());
    }
}
