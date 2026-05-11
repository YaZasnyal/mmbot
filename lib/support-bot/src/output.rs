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
    sanitized = strip_leading_thread_context_markers(sanitized);

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

fn strip_leading_thread_context_markers(mut value: String) -> String {
    loop {
        let trimmed_start = value.trim_start();
        let leading_whitespace = value.len() - trimmed_start.len();
        if !trimmed_start.starts_with("[post_id=") {
            return value;
        }

        let Some(end) = trimmed_start.find(']') else {
            return value;
        };
        let marker = &trimmed_start[..=end];
        if !is_thread_context_marker(marker) {
            return value;
        }

        value.drain(..leading_whitespace + end + 1);
    }
}

fn is_thread_context_marker(marker: &str) -> bool {
    marker.starts_with("[post_id=") && marker.contains(", user_id=") && marker.ends_with(']')
}

#[cfg(test)]
#[path = "tests/output.rs"]
mod tests;
