const HIDDEN_REASONING_TAGS: &[(&str, &str)] = &[
    ("<think>", "</think>"),
    ("<thinking>", "</thinking>"),
    ("<reasoning>", "</reasoning>"),
];

pub(crate) fn sanitize_user_visible_message(message: String) -> Option<String> {
    let mut sanitized = message;
    for (open, close) in HIDDEN_REASONING_TAGS {
        sanitized = strip_tagged_blocks(sanitized, open, close);
    }

    let sanitized = sanitized.trim().to_string();
    if sanitized.is_empty() {
        None
    } else {
        Some(sanitized)
    }
}

fn strip_tagged_blocks(mut value: String, open: &str, close: &str) -> String {
    loop {
        let lower = value.to_ascii_lowercase();
        let Some(start) = lower.find(open) else {
            return value;
        };
        let after_open = start + open.len();
        let Some(end_from_after_open) = lower[after_open..].find(close) else {
            value.truncate(start);
            return value;
        };
        let end = after_open + end_from_after_open + close.len();
        value.replace_range(start..end, "");
    }
}

#[cfg(test)]
#[path = "tests/output.rs"]
mod tests;
