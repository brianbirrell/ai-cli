#[cfg(test)]
mod integration_tests {
    use super::*;
    use mockito::{mock, Server};
    use std::io::Write;

    #[tokio::test]
    async fn test_integration_with_mock_server() {
        let mut server = Server::new();

        // Mock the API response
        let mock_response = r#"{"choices":[{"delta":{"content":"Test"},"finish_reason":null}]}"#;
        let _m = mock("POST", "/v1/chat/completions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&mock_response)
            .create();

        let args = Args {
            files: vec![],
            prompt: Some("Test".to_string()),
            model: "test-model".to_string(),
            base_url: server.url(),
        };

        // Create a temporary file for input
        let mut input_file = tempfile::NamedTempFile::new().unwrap();
        writeln!(input_file, "Test input").unwrap();

        let args = Args {
            files: vec![input_file.path().to_path_buf()],
            ..args
        };

        // Redirect stdout to capture output
        let stdout = std::io::stdout();
        let handle = stdout.lock();
        let mut output = Vec::<u8>::new();
        let mut output_write = std::io::BufWriter::new(&mut output);
        let _out = std::io::replace(&mut std::io::stdout(), output_write);

        // Run the main function
        main().await.expect("Main function should succeed");

        // Restore stdout
        std::io::stdout().write_all(&output).unwrap();

        // Verify output
        let output_str = std::str::from_utf8(&output).unwrap();
        assert!(output_str.contains("Test"));
    }
}