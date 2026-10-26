pub const LOG_BODY_MAX: usize = 64 * 1024;
pub const COLLECT_MAX: usize = 1024 * 1024;

pub fn truncate(value: &str, max_bytes: usize) -> String {
    if value.len() <= max_bytes {
        return value.to_string();
    }
    let mut end = max_bytes;
    while end > 0 && !value.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…[truncated]", &value[..end])
}
