use anyhow::{Context, Result};
use clap::Parser;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
};
use tokio::io::AsyncWriteExt;

/// OpenAI Compatible API Client
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file(s) to process
    #[arg(short, long)]
    files: Vec<PathBuf>,

    /// Prompt to provide context
    #[arg(short, long)]
    prompt: Option<String>,

    /// Model to use
    #[arg(short, long, default_value = "gpt-3.5-turbo")]
    model: String,

    /// Base URL for the API
    #[arg(long, default_value = "https://api.openai.com/v1")]
    base_url: String,

    /// API Key (if needed)
    #[arg(long)]
    api_key: Option<String>,
}

#[derive(Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<CompletionChoice>,
    // Other fields we might ignore
}

#[derive(Deserialize)]
struct CompletionChoice {
    delta: ChoiceDelta,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct ChoiceDelta {
    content: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let client = Client::builder()
        .build()?;

    // Read all input sources
    let input = read_input(&args).await?;

    // Build the request
    let request = ChatCompletionRequest {
        model: args.model,
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: input,
        }],
        stream: true,
    };

    // Send the request and stream the response
    stream_response(&client, &args.base_url, request).await?;
    println!(); // Print a newline at the end for clean output

    Ok(())
}

async fn read_input(args: &Args) -> Result<String> {
    let mut input = String::new();

    // Read from files if specified
    for file_path in &args.files {
        let mut file = File::open(file_path)
            .with_context(|| format!("Failed to open file: {}", file_path.display()))?;
        let mut file_content = String::new();
        file.read_to_string(&mut file_content)?;
        input.push_str(&file_content);
        input.push('\n');
    }

    // Read from stdin if not in pipe mode or if stdin is not empty
    if io::stdin().is_terminal() && args.files.is_empty() {
        // No files and stdin is tty, prompt for input
        io::stdout().write_all(b"Enter your input (Ctrl+D to finish):\n")?;
        io::stdout().flush()?;
        io::stdin()
            .read_to_string(&mut input)
            .with_context("Failed to read from stdin")?;
    } else {
        // Else read from stdin (could be pipe)
        io::stdin()
            .read_to_string(&mut input)
            .with_context("Failed to read from stdin")?;
    }

    // Add prompt if provided
    if let Some(prompt) = &args.prompt {
        input = format!("{} {}\n{}", "Prompt:", prompt, input);
    }

    Ok(input)
}

async fn stream_response(client: &Client, base_url: &str, request: ChatCompletionRequest) -> Result<()> {
    // Construct the full URL
    let url = if base_url.ends_with('/') {
        format!("{}/chat/completions", base_url.trim_end_matches('/'))
    } else {
        format!("{}/chat/completions", base_url)
    };

    // Add API key to headers if provided
    let mut request_builder = client
        .post(&url)
        .json(&request);

    if let Some(api_key) = request.api_key.as_ref() {
        request_builder = request_builder
            .header("Authorization", format!("Bearer {}", api_key));
    }

    let mut stream = request_builder
        .send()
        .await
        .with_context("Failed to send request")?
        .bytes_stream();

    let mut buffer = String::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.with_context("Failed to read response chunk")?;
        let text = std::str::from_utf8(&chunk)
            .with_context("Failed to decode response as UTF-8")?;
        buffer.push_str(text);

        // Try to parse the JSON lines
        if let Ok(response) = process_response_chunks(&buffer) {
            if let Some(choices) = response.into_choices() {
                for choice in choices {
                    if let Some(content) = choice.delta.and_then(|d| d.content) {
                        print!("{}", content);
                        io::stdout().flush()?;
                    }
                }
            }

            // Reset the buffer if we've processed everything
            if let Some(choice) = response.first_choice() {
                if choice.finish_reason.is_some() {
                    buffer.clear();
                }
            }
        }
    }

    Ok(())
}

// Helper function to process response chunks
fn process_response_chunks(buffer: &str) -> Result<ChatCompletionResponse> {
    let mut responses = vec![];
    let lines: Vec<&str> = buffer.lines().collect();

    for line in lines {
        if line.trim().is_empty()
            || line.starts_with("data: [DONE]")
            || !line.starts_with("data: ")
        {
            continue;
        }

        if let Ok(data) = line.trim_start_matches("data: ").to_string() {
            if !data.is_empty() {
                if let Ok(response) = serde_json::from_str::<ChatCompletionResponse>(&data) {
                    responses.push(response);
                }
            }
        }
    }

    if responses.is_empty() {
        anyhow::bail!("No valid response chunks found in buffer");
    }

    if responses.len() == 1 {
        Ok(responses.into_iter().next().unwrap())
    } else {
        Ok(concat_responses(responses))
    }
}

// Helper methods for ChatCompletionResponse
impl ChatCompletionResponse {
    fn into_choices(self) -> Option<Vec<CompletionChoice>> {
        if self.choices.is_empty() {
            None
        } else {
            Some(self.choices)
        }
    }

    fn first_choice(&self) -> Option<&CompletionChoice> {
        self.choices.first()
    }
}

// Helper function to concatenate responses
fn concat_responses(mut responses: Vec<ChatCompletionResponse>) -> ChatCompletionResponse {
    if responses.is_empty() {
        return ChatCompletionResponse {
            choices: vec![],
        };
    }

    // Take the first response
    let first = responses.remove(0);

    // All other choices should be combined into the first one
    for response in responses {
        if response.choices.is_empty() {
            continue;
        }

        let last_choice_index = first.choices.len() - 1;
        if last_choice_index >= 0 {
            // Merge the delta content
            if let Some(response_delta) = response.choices[0].delta.clone() {
                // This is a simplified merging strategy
                // In a real implementation, you'd need to handle this more carefully
                if let Some(content) = response_delta.content {
                    if let Some(first_choice) = first.choices.get_mut(last_choice_index) {
                        if let Some(first_content) = first_choice.delta.content.as_mut() {
                            first_content.push_str(&content);
                        } else {
                            first_choice.delta.content = Some(content);
                        }
                    }
                }
            }
        }
    }

    first
}