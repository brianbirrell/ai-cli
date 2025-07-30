use anyhow::{Context, Result};
use clap::Parser;
use reqwest::{Client};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    fs::File,
    io::{self, Read, Write, IsTerminal},
    path::PathBuf,
};
use tokio_stream::StreamExt;

// Configuration structure
#[derive(Debug, Serialize, Deserialize, Default)]
struct AppConfig {
    model: String,
    base_url: String,
    api_key: Option<String>,
    default_prompt: Option<String>,
}

impl AppConfig {
    fn default() -> Self {
        AppConfig {
            model: "llama3".to_string(),
            base_url: "http://localhost:11434/v1".to_string(),
            api_key: None,
            default_prompt: None,
        }
    }
}

/// OpenAI Compatible API Client
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // Now these are options to override the default config
    /// Input file(s) to process
    #[arg(short, long)]
    files: Vec<PathBuf>,

    /// Prompt to provide context
    #[arg(short, long)]
    prompt: Option<String>,

    /// Model to use
    #[arg(short, long)]
    model: Option<String>,

    /// Base URL for the API
    #[arg(long)]
    base_url: Option<String>,

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

#[derive(Deserialize, Clone)]
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
    // Get the final config to determine the model string
    let config = get_final_config(&args).await?;

    let request = ChatCompletionRequest {
        model: config.model.clone(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: input,
        }],
        stream: true,
    };

    // Send the request and stream the response, passing the api_key from args
    stream_response(&client, config.base_url.as_str(), args.api_key.as_ref(), request).await?;
    println!(); // Print a newline at the end for clean output

    Ok(())
}

// Load and merge configuration from file and command line
async fn get_final_config(args: &Args) -> Result<AppConfig> {
    // First load from config file
    let mut config = load_config()?;

    // Then override with command line arguments if provided
    if let Some(model) = &args.model {
        config.model = model.clone();
    }

    if let Some(base_url) = &args.base_url {
        config.base_url = base_url.clone();
    }

    if let Some(api_key) = &args.api_key {
        config.api_key = Some(api_key.clone());
    }

    Ok(config)
}

// Load configuration from config file
fn load_config() -> Result<AppConfig> {
    let config_dir = get_config_dir()?;
    let config_path = config_dir.join("config.toml");

    if !config_path.exists() {
        // Create default config if file doesn't exist
        create_default_config(&config_dir)?;
        return Ok(AppConfig::default());
    }

    // Read existing config file
    let config_contents = fs::read_to_string(&config_path)
        .context(format!("Failed to read config file: {}", config_path.display()))?;

    let config: AppConfig = toml::from_str(&config_contents)
        .context("Failed to parse config file as TOML")?;

    Ok(config)
}

// Create default config directory and file if they don't exist
fn create_default_config(config_dir: &PathBuf) -> Result<()> {
    // Create directory if it doesn't exist
    fs::create_dir_all(config_dir)
        .context("Failed to create config directory")?;

    // Create default config file
    let default_config = AppConfig::default();
    let toml_config = toml::to_string(&default_config)
        .context("Failed to serialize default config")?;

    fs::write(config_dir.join("config.toml"), toml_config)
        .context("Failed to write default config file")?;

    Ok(())
}

// Get or create config directory
fn get_config_dir() -> Result<PathBuf> {
    let home_dir = dirs::home_dir()
        .ok_or(anyhow::anyhow!("Could not find home directory"))?;

    let config_dir = home_dir.join(".config").join("ai-cli");
    Ok(config_dir)
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
            .with_context(|| String::from("Failed to read from stdin"))?;
    } else {
        // Else read from stdin (could be pipe)
        io::stdin()
            .read_to_string(&mut input)
            .with_context(|| String::from("Failed to read from stdin"))?;
    }

    // Add prompt if provided
    if let Some(prompt) = &args.prompt {
        input = format!("{} {}\n{}", "Prompt:", prompt, input);
    }

    Ok(input)
}

async fn stream_response(
    client: &Client,
    base_url: &str,
    api_key: Option<&String>,
    request: ChatCompletionRequest,
) -> Result<()> {
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

    if let Some(api_key) = api_key {
        request_builder = request_builder
            .header("Authorization", format!("Bearer {}", api_key));
    }

    let mut stream = request_builder
        .send()
        .await
        .with_context(|| String::from("Failed to send request"))?
        .bytes_stream();

    let mut incomplete = String::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.with_context(|| String::from("Failed to read response chunk"))?;
        let text = std::str::from_utf8(&chunk)
            .with_context(|| String::from("Failed to decode response as UTF-8"))?;

        incomplete.push_str(text);

        // Process complete lines only
        while let Some(pos) = incomplete.find('\n') {
            let line = incomplete[..pos].trim();
            if line.starts_with("data: ") && !line.starts_with("data: [DONE]") {
                let data = &line[6..];
                if !data.is_empty() {
                    if let Ok(response) = serde_json::from_str::<ChatCompletionResponse>(data) {
                        for choice in &response.choices {
                            if let Some(content) = choice.delta.content.as_ref() {
                                print!("{}", content);
                                io::stdout().flush()?;
                            }
                        }
                    }
                }
            }
            incomplete = incomplete[pos + 1..].to_string();
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

        let data = line.trim_start_matches("data: ").to_string();
        if !data.is_empty() {
            if let Ok(response) = serde_json::from_str::<ChatCompletionResponse>(&data) {
                responses.push(response);
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

// Helper function to concatenate responses
fn concat_responses(mut responses: Vec<ChatCompletionResponse>) -> ChatCompletionResponse {
    if responses.is_empty() {
        return ChatCompletionResponse {
            choices: vec![],
        };
    }

    // Take the first response
    let mut first = responses.remove(0);

    // All other choices should be combined into the first one
    for response in responses {
        if response.choices.is_empty() {
            continue;
        }

        let last_choice_index = first.choices.len() - 1;

        // Merge the delta content
        let response_delta = response.choices[0].delta.clone();
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

    first
}