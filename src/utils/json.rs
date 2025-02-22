use serde_json::Value;

// NOTE: I was not able to correctly parse serde_json::Value into a raw value
pub fn parse_serde_json_value_to_raw_string(v: &Value) -> String {
    let mut parsed_string = v.to_string();
    // Trim leading double quote
    if parsed_string.starts_with('"') {
        parsed_string.remove(0);
    }
    // Trim trailing double quote
    if parsed_string.ends_with('"') {
        parsed_string.pop();
    }
    // Reset string if escaped newline provided
    // This can occur when clicking on a cosmic::widget::text_editor widget
    if parsed_string == "\\n" {
        parsed_string = String::new();
    }
    // "Unescape" escaped new lines
    parsed_string.replace("\\n", "\n")
}
