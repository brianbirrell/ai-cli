use anyhow::{Context, Result};
use clap::Parser;
use log::{debug, info, trace};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    fs::File,
    io::{self, IsTerminal, Read, Write},
    path::PathBuf,
};
use tokio_stream::StreamExt;

// Build-time constants
const GIT_COMMIT_HASH: &str = env!("GIT_COMMIT_HASH", "unknown");
const GIT_COMMIT_HASH_SHORT: &str = env!("GIT_COMMIT_HASH_SHORT", "unknown");
const GIT_DIRTY: &str = env!("GIT_DIRTY", "unknown");
const BUILD_TIME: &str = env!("BUILD_TIME", "unknown");

pub fn print_version() {
    println!("ai-cli version {}", env!("CARGO_PKG_VERSION"));
    println!(
        "Commit: {}{}",
        GIT_COMMIT_HASH_SHORT,
        if GIT_DIRTY == "dirty" { "-dirty" } else { "" }
    );
    println!("Full commit: {GIT_COMMIT_HASH}");
    println!("Built: {BUILD_TIME}");
}

// Configuration structure
#[derive(Debug, Serialize, Deserialize, Default)]
struct AppConfig {
    model: String,
    base_url: String,
    api_key: Option<String>,
    default_prompt: Option<String>,
    temperature: Option<f32>,
    timeout_secs: u64,
}

impl AppConfig {
    fn default() -> Self {
        AppConfig {
            model: "llama3".to_string(),
            base_url: "http://localhost:11434/v1".to_string(),
            api_key: None,
            default_prompt: None,
            temperature: Some(0.7),
            timeout_secs: 300, // 300 seconds default timeout
        }
    }
}

/// OpenAI Compatible API Client
#[derive(Parser, Debug)]
#[command(author, about, long_about = None)]
pub struct Args {
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

    /// Enable verbose logging (use -v for basic debug, -vv for detailed request/response info)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Show version information
    #[arg(long)]
    version: bool,

    /// LLM temperature (0.0-2.0) - controls randomness
    #[arg(
        long,
        value_name = "FLOAT",
        help = "LLM temperature between 0.0 (deterministic) and 2.0 (creative)"
    )]
    temperature: Option<f32>,

    /// Connection timeout in seconds (applies only until first chunk arrives)
    #[arg(
        long,
        value_name = "SECS",
        help = "Connection timeout in seconds until first chunk (default: 300)"
    )]
    timeout: Option<u64>,
}

#[derive(Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
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
pub async fn main() -> Result<()> {
    let args = Args::parse();

    // Handle version flag
    if args.version {
        print_version();
        return Ok(());
    }

    // Initialize logging with appropriate verbosity
    let mut builder = env_logger::Builder::new();

    match args.verbose {
        0 => {
            // In normal mode, only show warnings and errors
            builder.filter_level(log::LevelFilter::Warn);
        }
        1 => {
            // In basic verbose mode (-v), show debug logs and above
            builder.filter_level(log::LevelFilter::Debug);
            debug!("Starting AI CLI application in basic verbose mode");
            debug!("Command line arguments: {args:?}");
        }
        2.. => {
            // In detailed verbose mode (-vv or more), show trace logs and above
            builder.filter_level(log::LevelFilter::Trace);
            trace!("Starting AI CLI application in detailed verbose mode");
            trace!("Command line arguments: {args:?}");
        }
    }

    builder.init();

    // Load and merge configuration from file and command line
    let config = get_final_config(&args).await?;
    let client = Client::builder().build()?;
    debug!("HTTP client initialized with no global timeout");

    // Read all input sources
    info!("Reading input from files and/or stdin");
    let input = read_input(&args).await?;
    debug!("Input length: {} characters", input.len());

    // Build the request
    info!("Building request with configuration");
    debug!(
        "Using model: {}, base_url: {}, temperature: {:?}, first-chunk timeout: {}s",
        config.model, config.base_url, config.temperature, config.timeout_secs
    );

    let request = ChatCompletionRequest {
        model: config.model.clone(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: input,
        }],
        stream: true,
        temperature: config.temperature,
    };
    debug!("Request prepared with streaming enabled");

    // Send the request and stream the response, passing the api_key from config or args
    info!("Sending request to API");
    stream_response(
        &client,
        config.base_url.as_str(),
        config.api_key.as_ref(),
        request,
        config.timeout_secs,
    )
    .await?;
    println!(); // Print a newline at the end for clean output
    info!("Response streaming completed");

    Ok(())
}

// Validate temperature is within acceptable range (0.0-2.0)
fn validate_temperature(temperature: f32) -> Result<f32> {
    if !(0.0..=2.0).contains(&temperature) {
        return Err(anyhow::anyhow!(
            "Temperature must be between 0.0 and 2.0, got: {}",
            temperature
        ));
    }
    Ok(temperature)
}

// Load and merge configuration from file and command line
async fn get_final_config(args: &Args) -> Result<AppConfig> {
    debug!("Loading configuration from file");
    // First load from config file
    let mut config = load_config()?;
    debug!("Base configuration loaded");

    // Then override with command line arguments if provided
    if let Some(model) = &args.model {
        debug!("Overriding model with command line argument: {model}");
        config.model = model.clone();
    }

    if let Some(base_url) = &args.base_url {
        debug!("Overriding base_url with command line argument: {base_url}");
        config.base_url = base_url.clone();
    }

    if let Some(api_key) = &args.api_key {
        debug!("Using API key from command line argument");
        config.api_key = Some(api_key.clone());
    }

    if let Some(temperature) = args.temperature {
        debug!("Overriding temperature with command line argument: {temperature}");
        config.temperature = Some(validate_temperature(temperature)?);
    } else if let Some(temperature) = config.temperature {
        // Validate the config file temperature as well
        config.temperature = Some(validate_temperature(temperature)?);
    }
    // If temperature is None in both config and args, leave it as None to use LLM default

    if let Some(timeout) = args.timeout {
        debug!("Overriding timeout with command line argument: {timeout}s");
        config.timeout_secs = timeout;
    }

    info!(
        "Final configuration: model={}, base_url={}, temperature={:?}, timeout={}s",
        config.model, config.base_url, config.temperature, config.timeout_secs
    );
    if config.api_key.is_some() {
        debug!("API key is configured");
    } else {
        debug!("No API key configured");
    }
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
    let config_contents = fs::read_to_string(&config_path).context(format!(
        "Failed to read config file: {}",
        config_path.display()
    ))?;

    let config: AppConfig =
        toml::from_str(&config_contents).context("Failed to parse config file as TOML")?;

    debug!("Configuration loaded successfully");
    Ok(config)
}

// Create default config directory and file if they don't exist
fn create_default_config(config_dir: &PathBuf) -> Result<()> {
    debug!("Creating config directory: {}", config_dir.display());
    // Create directory if it doesn't exist
    fs::create_dir_all(config_dir).context("Failed to create config directory")?;

    // Create default config file
    let default_config = AppConfig::default();
    let toml_config =
        toml::to_string(&default_config).context("Failed to serialize default config")?;

    let config_file_path = config_dir.join("config.toml");
    debug!("Writing default config to: {}", config_file_path.display());
    fs::write(config_file_path, toml_config).context("Failed to write default config file")?;

    info!("Default configuration created successfully");
    Ok(())
}

// Get or create config directory
fn get_config_dir() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or(anyhow::anyhow!("Could not find home directory"))?;

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
            debug!(
                "File {} read successfully, content length: {}",
                i + 1,
                file_content.len()
            );
        }
    }

    // Read from stdin if not in pipe mode or if stdin is not empty
    if io::stdin().is_terminal() && args.files.is_empty() {
        // No files and stdin is tty, prompt for input
        info!("Reading input from terminal (interactive mode)");
        io::stdout()
            .write_all(b"Enter the data you'd like the AI to work on (Ctrl+D to submit):\n")?;
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
        debug!("Adding prompt to input: {prompt}");
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
    first_chunk_timeout_secs: u64,
) -> Result<()> {
    // Construct the full URL
    let url = if base_url.ends_with('/') {
        format!("{}/chat/completions", base_url.trim_end_matches('/'))
    } else {
        format!("{base_url}/chat/completions")
    };
    debug!("API endpoint: {url}");

    // Add API key to headers if provided
    let mut request_builder = client.post(&url).json(&request);

    if let Some(api_key) = api_key {
        debug!("Adding API key to request headers");
        request_builder = request_builder.header("Authorization", format!("Bearer {api_key}"));
    } else {
        debug!("No API key provided - this may cause authentication errors if the API requires authentication");
    }

    // Enhanced logging for debugging - log full request details
    trace!("=== SERVICE CALL DETAILS ===");
    trace!("URL: {url}");
    trace!(
        "Request Body: {}",
        serde_json::to_string_pretty(&request)
            .unwrap_or_else(|_| "Failed to serialize request".to_string())
    );

    // Log headers that will be sent
    let mut headers_log = String::from("Headers: ");
    if let Some(_api_key) = api_key {
        headers_log.push_str("Authorization: Bearer ***, ");
    }
    headers_log.push_str("Content-Type: application/json");
    trace!("{headers_log}");
    trace!("=== END SERVICE CALL DETAILS ===");

    info!("Sending streaming request to API");
    debug!("Request builder prepared, sending HTTP POST request");
    let response = request_builder
        .send()
        .await
        .with_context(|| String::from("Failed to send request"))?;

    // Check and log the response status
    let status = response.status();
    info!(
        "API response status: {} ({})",
        status.as_u16(),
        status.canonical_reason().unwrap_or("Unknown")
    );

    if !status.is_success() {
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| "Unable to read error response body".to_string());
        return Err(anyhow::anyhow!(
            "API request failed with status {}: {}",
            status.as_u16(),
            error_body
        ));
    }

    debug!("API connection successful, starting to stream response");
    let mut stream = response.bytes_stream();

    let mut incomplete = String::new();
    let mut chunk_count = 0;

    info!("Starting to stream response");
    // Wait for the first chunk with timeout
    let first_chunk = tokio::time::timeout(
        std::time::Duration::from_secs(first_chunk_timeout_secs),
        stream.next(),
    )
    .await
    .with_context(|| String::from("Timed out waiting for the first response chunk"))?;

    if let Some(first_chunk_result) = first_chunk {
        chunk_count += 1;
        let chunk =
            first_chunk_result.with_context(|| String::from("Failed to read response chunk"))?;
        let text = std::str::from_utf8(&chunk)
            .with_context(|| String::from("Failed to decode response as UTF-8"))?;
        debug!("Received chunk {}: {} bytes", chunk_count, chunk.len());
        trace!("Chunk {chunk_count} content: {text:?}");
        incomplete.push_str(text);
    } else {
        return Err(anyhow::anyhow!("Stream ended before any data was received"));
    }

    // After the first chunk, continue without timeout
    while let Some(chunk) = stream.next().await {
        chunk_count += 1;
        let chunk = chunk.with_context(|| String::from("Failed to read response chunk"))?;
        let text = std::str::from_utf8(&chunk)
            .with_context(|| String::from("Failed to decode response as UTF-8"))?;

        debug!("Received chunk {}: {} bytes", chunk_count, chunk.len());
        trace!("Chunk {chunk_count} content: {text:?}");
        incomplete.push_str(text);

        // Process complete lines only
        while let Some(pos) = incomplete.find('\n') {
            let line = incomplete[..pos].trim();
            if line.starts_with("data: ") && !line.starts_with("data: [DONE]") {
                let data = &line[6..];
                if !data.is_empty() {
                    match serde_json::from_str::<ChatCompletionResponse>(data) {
                        Ok(response) => {
                            for choice in &response.choices {
                                if let Some(content) = choice.delta.content.as_ref() {
                                    print!("{content}");
                                    io::stdout().flush()?;
                                }
                            }
                        }
                        Err(e) => {
                            debug!("Failed to parse JSON response: {e}");
                            debug!("Raw data: {data}");
                        }
                    }
                }
            } else if line.starts_with("data: [DONE]") {
                debug!("Received end-of-stream marker");
            }
            incomplete = incomplete[pos + 1..].to_string();
        }
    }

    info!("Streaming completed after {chunk_count} chunks");
    debug!(
        "Final incomplete buffer length: {} characters",
        incomplete.len()
    );
    if !incomplete.is_empty() {
        debug!("Remaining incomplete data: {incomplete}");
    }
    Ok(())
}

#[cfg(test)]
mod unit_tests;
