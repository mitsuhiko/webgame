pub fn format_join_code(code: &str) -> String {
    let code = code.replace("-", "").to_ascii_uppercase();
    if code.len() > 3 {
        format!("{}-{}", &code[..3], &code[3..])
    } else {
        code.to_string()
    }
}
