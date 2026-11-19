pub const MAX_BLOCK_CHARS: usize = 1700;

pub fn code_block(text: &str, max_chars: usize) -> String {
    let text = text.trim_end().replace("```", "`\u{200b}``");
    let count = text.chars().count();

    let body = if count > max_chars {
        let tail: String = text.chars().skip(count - max_chars).collect();
        format!("…（前面省略 {} 字）\n{tail}", count - max_chars)
    } else {
        text
    };

    format!("```\n{body}\n```")
}
