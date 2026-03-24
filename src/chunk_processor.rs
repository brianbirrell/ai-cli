use crate::aggregation::build_aggregate_prompt;
use crate::input_stream::InputChunker;
use crate::{
    stream_response, stream_response_collect, AppConfig, Args, ChatCompletionRequest, ChatMessage,
    InputMode,
};
use anyhow::{Context, Result};
use log::debug;
use reqwest::Client;
use std::{
    fs,
    fs::File,
    io::{self, BufRead, BufReader, IsTerminal},
    time::Duration,
};

pub(crate) const DEFAULT_CHUNK_PROMPT_TEMPLATE: &str = "You are processing part {{chunk_index}}.\nFollow the user request:\n{{user_prompt}}\n\nPrevious summary:\n{{rolling_summary}}\n\nCurrent chunk:\n{{chunk_text}}";

pub(crate) fn should_use_chunked_mode(args: &Args, config: &AppConfig) -> Result<bool> {
    match config.input_mode {
        InputMode::Off => Ok(false),
        InputMode::Chunked => Ok(true),
        InputMode::Auto => {
            if !args.files.is_empty() {
                let mut total_bytes = 0usize;
                for file_path in &args.files {
                    let metadata = fs::metadata(file_path).with_context(|| {
                        format!("Failed to read file metadata: {}", file_path.display())
                    })?;
                    total_bytes = total_bytes.saturating_add(metadata.len() as usize);
                }
                Ok(total_bytes >= config.auto_chunk_threshold_chars)
            } else if !io::stdin().is_terminal() {
                // For piped stdin, we cannot cheaply know size up-front without buffering all input.
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }
}

pub(crate) fn render_chunk_prompt(
    template: &str,
    user_prompt: &str,
    rolling_summary: &str,
    chunk_text: &str,
    chunk_index: usize,
) -> String {
    template
        .replace("{{chunk_index}}", &chunk_index.to_string())
        .replace("{{user_prompt}}", user_prompt)
        .replace("{{rolling_summary}}", rolling_summary)
        .replace("{{chunk_text}}", chunk_text)
}

fn truncate_chars(input: &str, max_chars: usize) -> String {
    input.chars().take(max_chars).collect()
}

pub(crate) async fn process_large_input(
    args: &Args,
    config: &AppConfig,
    client: &Client,
    show_progress: bool,
) -> Result<()> {
    log::info!(
        "Processing large input in chunked mode with chunk_size={} and overlap={}",
        config.chunk_size_chars,
        config.chunk_overlap_chars
    );

    let user_prompt = args
        .prompt
        .clone()
        .or_else(|| config.default_prompt.clone())
        .unwrap_or_else(|| "Process the chunked input and provide useful output.".to_string());

    let chunk_template = if let Some(template_file) = &config.chunk_prompt_file {
        fs::read_to_string(template_file).with_context(|| {
            format!(
                "Failed to read chunk prompt template file: {}",
                template_file.display()
            )
        })?
    } else {
        DEFAULT_CHUNK_PROMPT_TEMPLATE.to_string()
    };

    let mut chunker = InputChunker::new(config.chunk_size_chars, config.chunk_overlap_chars);
    let mut rolling_summary = String::new();
    let mut chunk_index = 0usize;
    let mut aggregate_inputs: Vec<String> = Vec::new();

    if !args.files.is_empty() {
        for file_path in &args.files {
            let file = File::open(file_path)
                .with_context(|| format!("Failed to open file: {}", file_path.display()))?;
            let mut reader = BufReader::new(file);
            process_reader_chunks(
                &mut reader,
                &mut chunker,
                false,
                &chunk_template,
                &user_prompt,
                &mut rolling_summary,
                &mut chunk_index,
                &mut aggregate_inputs,
                config,
                client,
                show_progress,
            )
            .await?;
        }
    } else {
        let stdin = io::stdin();
        let mut reader = BufReader::new(stdin.lock());
        process_reader_chunks(
            &mut reader,
            &mut chunker,
            false,
            &chunk_template,
            &user_prompt,
            &mut rolling_summary,
            &mut chunk_index,
            &mut aggregate_inputs,
            config,
            client,
            show_progress,
        )
        .await?;
    }

    process_reader_chunks(
        &mut BufReader::new(io::empty()),
        &mut chunker,
        true,
        &chunk_template,
        &user_prompt,
        &mut rolling_summary,
        &mut chunk_index,
        &mut aggregate_inputs,
        config,
        client,
        show_progress,
    )
    .await?;

    if chunk_index == 0 {
        return Err(anyhow::anyhow!(
            "No input data was provided for chunked processing"
        ));
    }

    if config.aggregate_chunks && aggregate_inputs.len() > 1 {
        if args.verbose > 0 {
            println!("\n[aggregate] Generating final combined answer...\n");
        }

        let aggregate_prompt = build_aggregate_prompt(&user_prompt, &aggregate_inputs);

        let request = ChatCompletionRequest {
            model: config.model.clone(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: aggregate_prompt,
            }],
            stream: true,
            temperature: config.temperature,
        };

        stream_response(
            client,
            config.base_url.as_str(),
            config.api_key.as_ref(),
            request,
            config.timeout_secs,
            show_progress,
        )
        .await?;
    }

    println!();
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn process_reader_chunks<R: BufRead>(
    reader: &mut R,
    chunker: &mut InputChunker,
    flush_only: bool,
    chunk_template: &str,
    user_prompt: &str,
    rolling_summary: &mut String,
    chunk_index: &mut usize,
    aggregate_inputs: &mut Vec<String>,
    config: &AppConfig,
    client: &Client,
    show_progress: bool,
) -> Result<()> {
    if !flush_only {
        let mut line = String::new();
        loop {
            line.clear();
            let read_bytes = reader.read_line(&mut line)?;
            if read_bytes == 0 {
                break;
            }

            chunker.push_str(&line);
            while let Some(chunk_text) = chunker.next_chunk(false) {
                *chunk_index += 1;
                process_single_chunk(
                    *chunk_index,
                    &chunk_text,
                    chunk_template,
                    user_prompt,
                    rolling_summary,
                    aggregate_inputs,
                    config,
                    client,
                    show_progress,
                )
                .await?;
                if config.max_chunks > 0 && *chunk_index >= config.max_chunks {
                    return Err(anyhow::anyhow!(
                        "Reached max_chunks limit ({})",
                        config.max_chunks
                    ));
                }
            }
        }
    }

    while let Some(chunk_text) = chunker.next_chunk(true) {
        *chunk_index += 1;
        process_single_chunk(
            *chunk_index,
            &chunk_text,
            chunk_template,
            user_prompt,
            rolling_summary,
            aggregate_inputs,
            config,
            client,
            show_progress,
        )
        .await?;
        if config.max_chunks > 0 && *chunk_index >= config.max_chunks {
            return Err(anyhow::anyhow!(
                "Reached max_chunks limit ({})",
                config.max_chunks
            ));
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn process_single_chunk(
    chunk_index: usize,
    chunk_text: &str,
    chunk_template: &str,
    user_prompt: &str,
    rolling_summary: &mut String,
    aggregate_inputs: &mut Vec<String>,
    config: &AppConfig,
    client: &Client,
    show_progress: bool,
) -> Result<()> {
    if log::log_enabled!(log::Level::Debug) {
        debug!(
            "Processing chunk {} ({} chars)",
            chunk_index,
            chunk_text.chars().count()
        );
    }

    let rendered_prompt = render_chunk_prompt(
        chunk_template,
        user_prompt,
        rolling_summary,
        chunk_text,
        chunk_index,
    );

    let request = ChatCompletionRequest {
        model: config.model.clone(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: rendered_prompt,
        }],
        stream: true,
        temperature: config.temperature,
    };

    let chunk_output = stream_response_with_retries(
        client,
        config.base_url.as_str(),
        config.api_key.as_ref(),
        request,
        config.timeout_secs,
        chunk_index,
        show_progress,
    )
    .await?;

    *rolling_summary = truncate_chars(&chunk_output, 2000);
    if config.aggregate_chunks {
        aggregate_inputs.push(truncate_chars(&chunk_output, 1500));
    }

    Ok(())
}

async fn stream_response_with_retries(
    client: &Client,
    base_url: &str,
    api_key: Option<&String>,
    request: ChatCompletionRequest,
    first_chunk_timeout_secs: u64,
    chunk_index: usize,
    show_progress: bool,
) -> Result<String> {
    let max_retries = 2;
    let mut backoff_secs = 1u64;

    for attempt in 0..=max_retries {
        let request_for_attempt = ChatCompletionRequest {
            model: request.model.clone(),
            messages: request.messages.clone(),
            stream: request.stream,
            temperature: request.temperature,
        };

        match stream_response_collect(
            client,
            base_url,
            api_key,
            request_for_attempt,
            first_chunk_timeout_secs,
            true,
            show_progress,
        )
        .await
        {
            Ok(output) => return Ok(output),
            Err(err) => {
                if attempt == max_retries {
                    return Err(anyhow::anyhow!(
                        "Chunk {} failed after {} attempt(s): {}",
                        chunk_index,
                        max_retries + 1,
                        err
                    ));
                }
                debug!(
                    "Chunk {} attempt {} failed, retrying in {}s: {}",
                    chunk_index,
                    attempt + 1,
                    backoff_secs,
                    err
                );
                tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
                backoff_secs = backoff_secs.saturating_mul(2);
            }
        }
    }

    Err(anyhow::anyhow!(
        "Unexpected retry loop exit for chunk {}",
        chunk_index
    ))
}
