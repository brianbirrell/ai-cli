pub(crate) fn build_aggregate_prompt(user_prompt: &str, chunk_outputs: &[String]) -> String {
    format!(
        "User request:\n{}\n\nCombine the following per-chunk outputs into a single coherent final answer:\n\n{}",
        user_prompt,
        chunk_outputs.join("\n\n")
    )
}
