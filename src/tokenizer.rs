//! Tokenizer for CIF format
//! Handles line splitting and quoted string detection

/// Split a line into tokens, handling quoted strings correctly.
/// Returns Vec of (token_string, is_quoted)
pub fn special_split(content: &str) -> Vec<(String, bool)> {
    let mut output: Vec<(String, bool)> = Vec::new();
    let mut current = String::new();
    let mut current_quoted = false;
    let mut in_quote = false;
    let mut quote_char: Option<char> = None;
    
    let chars: Vec<char> = content.chars().collect();
    let len = chars.len();
    
    for i in 0..len {
        let c = chars[i];
        let is_ws = c == ' ' || c == '\t';
        
        // Check for quote boundaries
        if (c == '\'' || c == '"') && !in_quote {
            // Start quote if at boundary
            let at_start = i == 0 || chars[i - 1] == ' ' || chars[i - 1] == '\t';
            if at_start {
                in_quote = true;
                quote_char = Some(c);
                current.push(c);
                current_quoted = true;
                continue;
            }
        } else if in_quote && Some(c) == quote_char {
            // End quote if at boundary
            let at_end = i == len - 1 || chars.get(i + 1).map_or(true, |&nc| nc == ' ' || nc == '\t');
            if at_end {
                current.push(c);
                in_quote = false;
                quote_char = None;
                continue;
            }
        }
        
        if !in_quote && is_ws {
            // Whitespace outside quote - end current token
            if !current.is_empty() {
                output.push((current, current_quoted));
                current = String::new();
                current_quoted = false;
            }
        } else if !in_quote && c == '#' {
            // Comment - stop processing
            break;
        } else {
            // Regular character
            current.push(c);
        }
    }
    
    // Push final token if non-empty
    if !current.is_empty() {
        output.push((current, current_quoted));
    }
    
    output
}

/// Check if a token is a data name (starts with _ and is not quoted)
pub fn is_data_name(token: &str, is_quoted: bool) -> bool {
    !is_quoted && token.starts_with('_')
}

/// Check if a token is a loop keyword
pub fn is_loop_keyword(token: &str, is_quoted: bool) -> bool {
    !is_quoted && token == "loop_"
}

/// Check if a token is a data/save/global keyword
pub fn is_block_keyword(token: &str, is_quoted: bool) -> bool {
    if is_quoted {
        return false;
    }
    token == "global_" || token.starts_with("data_") || token.starts_with("save_")
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_split() {
        let result = special_split("foo bar baz");
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], ("foo".to_string(), false));
        assert_eq!(result[1], ("bar".to_string(), false));
        assert_eq!(result[2], ("baz".to_string(), false));
    }
    
    #[test]
    fn test_quoted_string() {
        let result = special_split("'hello world' test");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], ("'hello world'".to_string(), true));
        assert_eq!(result[1], ("test".to_string(), false));
    }
    
    #[test]
    fn test_comment() {
        let result = special_split("foo bar # comment");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], ("foo".to_string(), false));
        assert_eq!(result[1], ("bar".to_string(), false));
    }
    
    #[test]
    fn test_data_name() {
        assert!(is_data_name("_atom_site.id", false));
        assert!(!is_data_name("_atom_site.id", true));
        assert!(!is_data_name("atom_site", false));
    }
    
    #[test]
    fn test_keywords() {
        assert!(is_loop_keyword("loop_", false));
        assert!(!is_loop_keyword("loop_", true));
        
        assert!(is_block_keyword("data_1ABC", false));
        assert!(is_block_keyword("save_test", false));
        assert!(is_block_keyword("global_", false));
        assert!(!is_block_keyword("data_1ABC", true));
    }
}
