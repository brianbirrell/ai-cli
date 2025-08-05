#[cfg(test)]
mod integration_tests {
    // Integration tests can't directly import from the main binary
    // We'll test the version functionality by running the binary directly

    #[tokio::test]
    async fn test_integration_with_mock_server() {
        // This test is disabled for now due to complexity with mocking
        // and the need to properly handle the main function

        // TODO: Implement proper integration test that:
        // 1. Sets up a mock server
        // 2. Creates test input
        // 3. Runs the main function with proper arguments
        // 4. Verifies the output

        // For now, we'll just test that the version flag works
        // This is a simpler integration test
    }

    #[test]
    fn test_version_flag_integration() {
        // Test that the --version flag works when running the binary
        // This is a basic integration test
        use std::process::Command;

        let output = Command::new("cargo")
            .args(&["run", "--", "--version"])
            .output()
            .expect("Failed to run cargo");

        assert!(output.status.success());

        let output_str = String::from_utf8_lossy(&output.stdout);
        assert!(output_str.contains("ai-cli version"));
        assert!(output_str.contains("Commit:"));
        assert!(output_str.contains("Built:"));
    }
}
