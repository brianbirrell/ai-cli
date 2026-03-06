pub(crate) struct InputChunker {
    chunk_size_chars: usize,
    chunk_overlap_chars: usize,
    buffer: String,
}

impl InputChunker {
    pub(crate) fn new(chunk_size_chars: usize, chunk_overlap_chars: usize) -> Self {
        Self {
            chunk_size_chars,
            chunk_overlap_chars,
            buffer: String::new(),
        }
    }

    pub(crate) fn push_str(&mut self, text: &str) {
        self.buffer.push_str(text);
    }

    pub(crate) fn next_chunk(&mut self, flush: bool) -> Option<String> {
        let buffer_chars = self.buffer.chars().count();
        if buffer_chars == 0 {
            return None;
        }
        if !flush && buffer_chars < self.chunk_size_chars {
            return None;
        }

        let take_chars = buffer_chars.min(self.chunk_size_chars);
        let chunk: String = self.buffer.chars().take(take_chars).collect();

        let step_chars = if flush && buffer_chars <= self.chunk_size_chars {
            buffer_chars
        } else {
            take_chars.saturating_sub(self.chunk_overlap_chars)
        };

        if step_chars == 0 {
            // Should never happen due to validation, but this prevents an infinite loop.
            self.buffer.clear();
            return Some(chunk);
        }

        self.buffer = self.buffer.chars().skip(step_chars).collect();
        Some(chunk)
    }
}
