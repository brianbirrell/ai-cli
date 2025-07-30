use super::*;
use std::io::Read;

#[tokio::test]
async fn test_read_empty_stdinput() {
    let args = Args {
        files: vec![],
        prompt: None,
        model: String::new(),
        base_url: String::new(),
    };

    // Test reading from empty stdin
    let input = read_input(&args).await.unwrap();
    assert!(input.is_empty());
}

#[tokio::test]
async fn test_read_from_file() {
    let tmpfile = tempfile::NamedTempFile::new().unwrap();
    writeln!(tmpfile, "This is test content").unwrap();

    let args = Args {
        files: vec![tmpfile.path().to_path_buf()],
        prompt: None,
        model: String::new(),
        base_url: String::new(),
    };

    let input = read_input(&args).await.unwrap();
    assert_eq!(input.trim(), "This is test content");
}

#[tokio::test]
async fn test_read_with_prompt() {
    let args = Args {
        files: vec![],
        prompt: Some("Test prompt".to_string()),
        model: String::new(),
        base_url: String::new(),
    };

    // Mock stdin
    let stdin = "test input".as_bytes();
    let mut stdin_wrapper = Vec::new();
    stdin_wrapper.extend_from_slice(stdin);

    let stdin_handle = GeneticStdIn::new(stdin_wrapper.as_slice());
    std::env::set_var("AI_CLI_TEST_STDIN", stdin_handle);

    let input = read_input(&args).await.unwrap();
    assert!(input.starts_with("Prompt: Test prompt"));
    assert!(input.contains("test input"));
}