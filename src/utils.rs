pub fn is_valid_field(field: &str) -> bool {
    !field.is_empty()
}

pub fn is_valid_key_field(field: &str) -> bool {
    !field.is_empty() && !field.contains(':')
}
