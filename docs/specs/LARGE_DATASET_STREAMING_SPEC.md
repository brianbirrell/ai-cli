# Large Dataset Streaming Specification

## Summary
Add support for processing large datasets without loading all input into memory or requiring a single oversized prompt. The CLI will ingest data as a stream, split it into bounded chunks, and send chunked requests to the LLM while preserving useful context between chunks.

## Problem Statement
Current behavior in `src/main.rs` reads all files and/or stdin into one `String` before sending one chat completion request. This fails for large inputs because:
- Memory usage grows with total input size.
- Single-request payloads can exceed model/context limits.
- Long-running pipe streams are not practical with full buffering.

## Goals
- Process arbitrarily large stdin/file input with bounded memory usage.
- Preserve the existing streaming response UX for generated output.
- Support chunked LLM processing with controllable context carryover.
- Provide deterministic behavior and clear progress logs in verbose mode.

## Non-Goals
- True token-by-token upload into one OpenAI chat completion request (most compatible endpoints require full request payload before generation).
- Automatic semantic chunking by document structure in v1.
- Distributed processing or parallel fan-out in v1.

## User Experience
### Configuration-only options
Chunking behavior is configured in `~/.config/ai-cli/config.toml`, with one CLI override for mode selection.

Command-line mode override:
- `-i, --input-mode <off|chunked|auto>` (default: `auto`)

Proposed config keys:
- `input_mode = "auto" | "off" | "chunked"` (default: `auto`)
- `chunk_size_chars = 16000`
- `chunk_overlap_chars = 1000`
- `max_chunks = 0` (`0` means unlimited)
- `auto_chunk_threshold_chars = 50000`
- `aggregate_chunks = true`
- `chunk_prompt_file = "/absolute/or/home-relative/path"` (optional)

Precedence:
- If `--input-mode` is provided, it overrides `input_mode` from config.
- All other chunk settings remain config-only in v1.

### Behavior
- `off`: existing behavior (single in-memory prompt).
- `chunked`: force stream ingestion + chunked requests.
- `auto`: if input exceeds threshold (`auto_chunk_threshold_chars`, default `50000`), switch to chunked mode.

### Output mode
- For each chunk, stream assistant output to stdout as it arrives.
- Prefix chunk boundaries in verbose mode only (not in normal mode), for example: `[chunk 3/12]`.
- If `aggregate_chunks = true`, run a final request over accumulated chunk summaries and stream the final answer.

## Functional Requirements
- Input reading must be incremental using buffered readers.
- Memory usage must be bounded to approximately `chunk_size + overlap + small overhead`.
- Each chunk request must include:
- User prompt (`--prompt`)
- Current chunk text
- Optional rolling summary from previous chunks
- Chunking controls must come from config file values, not CLI flags.
- On transient HTTP failures, retry chunk request up to 2 times with exponential backoff.
- On hard failures, exit non-zero and print chunk index and error context.

## Architecture Changes
### New modules
- `src/input_stream.rs`
- Stream data from files/stdin via `BufRead`.
- Yield `String` chunks by character boundary with overlap.
- `src/chunk_processor.rs`
- Build per-chunk chat requests.
- Execute chunk request via existing `stream_response` infrastructure (or extracted lower-level request function).
- `src/aggregation.rs`
- Optional final synthesis request over chunk outputs/summaries.

### Main flow updates (`src/main.rs`)
- Add one new argument: `-i, --input-mode <off|chunked|auto>`.
- Extend `AppConfig` with large-input options.
- Route execution:
- Small/single mode: existing path.
- Chunked mode: `process_large_input()` pipeline.

### Prompt template format
Default per-chunk prompt:
```
You are processing part {{chunk_index}} of {{chunk_count_estimate}}.
Follow the user request:
{{user_prompt}}

Previous summary:
{{rolling_summary}}

Current chunk:
{{chunk_text}}
```

## Error Handling
- Input read errors: include file path and byte offset when possible.
- Chunk serialization/request errors: include chunk index.
- Aggregation failure: print warning and still return chunk-level output unless `--aggregate-required` is set.

## Observability
- `-v`: log mode selection, chunk counts, chunk sizes, retries.
- `-vv`: additionally log truncated chunk metadata and request payload sizes (never raw API key).

## Backward Compatibility
- Existing commands continue to work unchanged when `input_mode = "off"` is explicitly configured.
- Existing commands continue to work unchanged without `--input-mode`.
- Default behavior is `auto`; single-request behavior is still used for small input until `auto_chunk_threshold_chars` is exceeded.

## Testing Plan
### Unit tests
- Chunk splitter emits expected sizes and overlap behavior.
- Auto-mode threshold selection.
- Prompt template rendering for chunk requests.

### Integration tests (`tests/integration_tests.rs`)
- Pipe a large generated input stream and verify:
- Process does not OOM for test-sized data.
- Multiple chunk requests are issued (via mocked server).
- Output ordering remains chunk-sequential.

### Regression tests
- Existing single-request behavior unchanged for small inputs.
- Temperature, timeout, and auth headers still applied in chunked mode.

## Acceptance Criteria
- CLI can process input at least 10x larger than prior practical limit without full-input buffering.
- Memory usage remains bounded and does not scale linearly with full input size.
- Chunked processing works for stdin pipes and multi-file input.
- Existing standard usage examples still pass.

## Implementation Milestones
1. Add config file fields and CLI mode override (`--input-mode`).
2. Implement buffered chunking reader.
3. Implement sequential chunk request execution with retries.
4. Add optional aggregation pass.
5. Add tests and update README/config example usage.
