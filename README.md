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
ai-cli -v -p "Test prompt"

# Show version information
ai-cli --version

# Custom API endpoint
ai-cli --base-url "https://api.openai.com/v1" --api-key "your-key" -p "Hello"
```

**Command line options:**  
All options are optional _except_ for `-p, --prompt`, which is required.

- `-p, --prompt <prompt>` (**required**): User prompt (can be combined with file/stdin input)
- `-m, --model <model>` (optional): LLM model to use (default: llama3)
- `-f, --files <file>` (optional): One or more files to send as input
- `--base-url <url>` (optional): API endpoint (default: http://localhost:11434/v1)
- `--api-key <key>` (optional): API key for authentication, if needed
- `-v, --verbose` (optional): Enable verbose logging
- `--version` (optional): Show version information

## Configuration

You can set defaults in `~/.config/ai-cli/config.toml`. The config file is automatically created on first run.

Example config file:
```toml
model = "llama3"
base_url = "http://localhost:11434/v1"
api_key = "your-api-key-here"
default_prompt = "You are a helpful assistant."
```

Command-line arguments will override config file values.

# Contributing

Contributions, issues, and feature requests are welcome! Feel free to open an issue or submit a pull request.

# License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
