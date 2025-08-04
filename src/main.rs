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
use log::{info, debug, warn, error};

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

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
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
}

#[derive(Deserialize, Clone)]
struct ChoiceDelta {
    content: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Initialize logging
    let log_level = if args.verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };
    
    env_logger::Builder::new()
        .filter_level(log_level)
        .init();
    
    info!("Starting AI CLI application");
    debug!("Command line arguments: {:?}", args);
    
    let client = Client::builder()
        .build()?;
    debug!("HTTP client initialized");

    // Read all input sources
    info!("Reading input from files and/or stdin");
    let input = read_input(&args).await?;
    debug!("Input length: {} characters", input.len());

    // Build the request
    // Get the final config to determine the model string
    info!("Loading configuration");
    let config = get_final_config(&args).await?;
    debug!("Using model: {}, base_url: {}", config.model, config.base_url);

    let request = ChatCompletionRequest {
        model: config.model.clone(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: input,
        }],
        stream: true,
    };
    debug!("Request prepared with streaming enabled");

    // Send the request and stream the response, passing the api_key from args
    info!("Sending request to API");
    stream_response(&client, config.base_url.as_str(), args.api_key.as_ref(), request).await?;
    println!(); // Print a newline at the end for clean output
    info!("Response streaming completed");

    Ok(())
}

// Load and merge configuration from file and command line
async fn get_final_config(args: &Args) -> Result<AppConfig> {
    debug!("Loading configuration from file");
    // First load from config file
    let mut config = load_config()?;
    debug!("Base configuration loaded");

    // Then override with command line arguments if provided
    if let Some(model) = &args.model {
        debug!("Overriding model with command line argument: {}", model);
        config.model = model.clone();
    }

    if let Some(base_url) = &args.base_url {
        debug!("Overriding base_url with command line argument: {}", base_url);
        config.base_url = base_url.clone();
    }

    if let Some(api_key) = &args.api_key {
        debug!("Using API key from command line argument");
        config.api_key = Some(api_key.clone());
    }

    info!("Final configuration: model={}, base_url={}", config.model, config.base_url);
    Ok(config)
}

// Load configuration from config file
fn load_config() -> Result<AppConfig> {
    let config_dir = get_config_dir()?;
    let config_path = config_dir.join("config.toml");
    debug!("Config path: {}", config_path.display());

    if !config_path.exists() {
        // Create default config if file doesn't exist
        info!("Config file not found, creating default configuration");
        create_default_config(&config_dir)?;
        return Ok(AppConfig::default());
    }

    // Read existing config file
    info!("Reading existing config file");
    let config_contents = fs::read_to_string(&config_path)
        .context(format!("Failed to read config file: {}", config_path.display()))?;

    let config: AppConfig = toml::from_str(&config_contents)
        .context("Failed to parse config file as TOML")?;

    debug!("Configuration loaded successfully");
    Ok(config)
}

// Create default config directory and file if they don't exist
fn create_default_config(config_dir: &PathBuf) -> Result<()> {
    debug!("Creating config directory: {}", config_dir.display());
    // Create directory if it doesn't exist
    fs::create_dir_all(config_dir)
        .context("Failed to create config directory")?;

    // Create default config file
    let default_config = AppConfig::default();
    let toml_config = toml::to_string(&default_config)
        .context("Failed to serialize default config")?;

    let config_file_path = config_dir.join("config.toml");
    debug!("Writing default config to: {}", config_file_path.display());
    fs::write(config_file_path, toml_config)
        .context("Failed to write default config file")?;

    info!("Default configuration created successfully");
    Ok(())
}

// Get or create config directory
fn get_config_dir() -> Result<PathBuf> {
    let home_dir = dirs::home_dir()
        .ok_or(anyhow::anyhow!("Could not find home directory"))?;

    let config_dir = home_dir.join(".config").join("ai-cli");
    debug!("Config directory: {}", config_dir.display());
    Ok(config_dir)
}

async fn read_input(args: &Args) -> Result<String> {
    let mut input = String::new();

    // Read from files if specified
    if !args.files.is_empty() {
        info!("Reading input from {} file(s)", args.files.len());
        for (i, file_path) in args.files.iter().enumerate() {
            debug!("Reading file {}: {}", i + 1, file_path.display());
            let mut file = File::open(file_path)
                .with_context(|| format!("Failed to open file: {}", file_path.display()))?;
            let mut file_content = String::new();
            file.read_to_string(&mut file_content)?;
            input.push_str(&file_content);
            input.push('\n');
            debug!("File {} read successfully, content length: {}", i + 1, file_content.len());
        }
    }

    // Read from stdin if not in pipe mode or if stdin is not empty
    if io::stdin().is_terminal() && args.files.is_empty() {
        // No files and stdin is tty, prompt for input
        info!("Reading input from terminal (interactive mode)");
        io::stdout().write_all(b"Enter your input (Ctrl+D to finish):\n")?;
        io::stdout().flush()?;
        io::stdin()
            .read_to_string(&mut input)
            .with_context(|| String::from("Failed to read from stdin"))?;
        debug!("Interactive input received, length: {}", input.len());
    } else if !args.files.is_empty() {
        // Files were provided, skip stdin
        debug!("Skipping stdin as files were provided");
    } else {
        // Else read from stdin (could be pipe)
        info!("Reading input from stdin (pipe mode)");
        io::stdin()
            .read_to_string(&mut input)
            .with_context(|| String::from("Failed to read from stdin"))?;
        debug!("Pipe input received, length: {}", input.len());
    }

    // Add prompt if provided
    if let Some(prompt) = &args.prompt {
        debug!("Adding prompt to input: {}", prompt);
        input = format!("{} {}\n{}", "Prompt:", prompt, input);
    }

    info!("Total input length: {} characters", input.len());
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
    debug!("API endpoint: {}", url);

    // Add API key to headers if provided
    let mut request_builder = client
        .post(&url)
        .json(&request);

    if let Some(api_key) = api_key {
        debug!("Adding API key to request headers");
        request_builder = request_builder
            .header("Authorization", format!("Bearer {}", api_key));
    } else {
        debug!("No API key provided");
    }

    info!("Sending streaming request to API");
    let mut stream = request_builder
        .send()
        .await
        .with_context(|| String::from("Failed to send request"))?
        .bytes_stream();

    let mut incomplete = String::new();
    let mut chunk_count = 0;

    info!("Starting to stream response");
    while let Some(chunk) = stream.next().await {
        chunk_count += 1;
        let chunk = chunk.with_context(|| String::from("Failed to read response chunk"))?;
        let text = std::str::from_utf8(&chunk)
            .with_context(|| String::from("Failed to decode response as UTF-8"))?;

        debug!("Received chunk {} ({} bytes)", chunk_count, chunk.len());
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
                                debug!("Outputting content: {}", content);
                                print!("{}", content);
                                io::stdout().flush()?;
                            }
                        }
                    }
                }
            } else if line.starts_with("data: [DONE]") {
                debug!("Received end-of-stream marker");
            }
            incomplete = incomplete[pos + 1..].to_string();
        }
    }

    info!("Streaming completed after {} chunks", chunk_count);
    Ok(())
}