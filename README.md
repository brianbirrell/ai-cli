# ai-cli

A command-line OpenAI-compatible API client, written in Rust, for interacting with chat completion models. This tool enables you to send prompts or file content to an LLM endpoint and stream the generated responses directly in your terminal.

# About The Project

**ai-cli** is designed for users who want a fast, scriptable, and fully configurable CLI interface to LLMs (e.g., Llama 3, local models, or OpenAI-compatible APIs). It supports streaming chat completions, flexible configuration, and easy integration into your development workflows.

## Features

- 📝 Send prompts from files or stdin directly to a chat completion API.
- ⚡ Stream LLM responses live in your terminal.
- 🔑 Supports configurable model, API base URL, and API keys.
- 🛠️ Automatically manages a config file in `~/.config/ai-cli/config.toml`.
- 🧩 Easily override config values from the command line.
- 🐧 Designed for local or remote OpenAI-compatible endpoints.
- 📂 Reads multiple files and combines their content as input.
- 🔄 Integrates smoothly into shell scripts and pipelines.
- 📊 Verbose logging for debugging and monitoring.
- ℹ️ Version information display.

## Project like this one

See also: [shell_gpt](https://github.com/TheR1D/shell_gpt), [ai-shell](https://github.com/BuilderIO/ai-shell)

## What Is New In v0.3.0

- Added large-input chunked processing with configurable input modes (`off`, `chunked`, `auto`)
- Added activity spinner improvements and `--no-progress` option for script-friendly output
- Added automated release workflow and Linux release packaging artifacts (`tar.gz`, `zip`, `deb`)
- Updated CI workflows and dependencies for security and compatibility

## Release Automation

This repository includes a GitHub Actions workflow that automates releases end to end:

- Validates the provided SemVer release version
- Updates `Cargo.toml` and `Cargo.lock` to the requested version
- Runs project validation checks in CI
- Pushes the release commit and creates a GitHub release tag in the format `vX.X.X`

Run the `Automate Release` workflow from the Actions tab with a SemVer version input (for example, `0.3.0`).

Current release: `v0.3.0`.

For full details and required token/repo settings, see [docs/VERSION_MANAGEMENT.md](docs/VERSION_MANAGEMENT.md).

# Getting Started

## Prerequisites

- Install Rust: [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)
  
- On Ubuntu/Debian:
  
  ```sh
  sudo apt update
  sudo apt install build-essential libssl-dev pkg-config
  ```
  

## Install

### Option 1: Build from source

Clone the repository and build with Cargo:

```sh
git clone https://github.com/brianbirrell/ai-cli.git
cd ai-cli
cargo build --release
```

The binary will be found at `target/release/ai-cli`.

### Option 2: Download pre-built release

Check the [releases page](https://github.com/brianbirrell/ai-cli/releases) for pre-built binaries.

# Usage

```sh
# Basic usage with prompt
ai-cli -p "Explain quantum computing in simple terms"

# Using stdin
cat myfile.txt | ai-cli -m llama3 -p "What is in this file?"

# Multiple files
ai-cli -f notes.txt -f summary.txt -p "Summarize these notes"

# Verbose output for debugging
ai-cli -v -p "Test prompt"          # Basic debug info
ai-cli -vv -p "Test prompt"         # Detailed request/response info

# Show version information
ai-cli --version

# Custom API endpoint
ai-cli --base-url "https://api.openai.com/v1" --api-key "your-key" -p "Hello"

# Control temperature and timeout
ai-cli --temperature 0.3 --timeout 60 -p "Write a creative story"

# Control input processing mode
ai-cli -i auto -p "Summarize this data" < large_input.txt
```

**Command line options:**  
All options are optional _except_ for `-p, --prompt`, which is required.

- `-p, --prompt <prompt>` (**required**): User prompt (can be combined with file/stdin input)
- `-m, --model <model>` (optional): LLM model to use (default: llama3)
- `-f, --files <file>` (optional): One or more files to send as input
- `--base-url <url>` (optional): API endpoint (default: http://localhost:11434/v1)
- `--api-key <key>` (optional): API key for authentication, if needed
- `--temperature <float>` (optional): LLM temperature between 0.0 (deterministic) and 2.0 (creative)
- `--timeout <secs>` (optional): Connection timeout in seconds until first chunk (default: 300)
- `-i, --input-mode <off|chunked|auto>` (optional): Input processing mode (default: auto)
- `--no-progress` (optional): Disable the activity spinner (useful for cron jobs and scripts)
- `-v, --verbose` (optional): Enable verbose logging (use -v for basic debug, -vv for detailed request/response info)
- `--version` (optional): Show version information

## Configuration

You can set defaults in `~/.config/ai-cli/config.toml`. The config file is automatically created on first run.

Example config file:
```toml
model = "llama3"
base_url = "http://localhost:11434/v1"
api_key = "your-api-key-here"
default_prompt = "You are a helpful assistant."
temperature = 0.7  # Optional: omit to use LLM's default temperature
timeout_secs = 300  # Optional: connection timeout in seconds (default: 300)
input_mode = "auto"  # Optional: off, chunked, auto
chunk_size_chars = 16000  # Optional: chunk size for large input mode
chunk_overlap_chars = 1000  # Optional: overlap between chunks
max_chunks = 0  # Optional: 0 means unlimited
auto_chunk_threshold_chars = 50000  # Optional: auto mode threshold
aggregate_chunks = true  # Optional: run final synthesis pass over chunk outputs
# chunk_prompt_file = "~/path/to/chunk_prompt.txt"  # Optional: custom chunk prompt template
# no_progress = false  # Optional: set to true to disable the activity spinner globally
```

Command-line arguments will override config file values.

# Contributing

Contributions, issues, and feature requests are welcome! Feel free to open an issue or submit a pull request.

# License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
