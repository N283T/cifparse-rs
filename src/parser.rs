//! CIF parser implementation
//! Parses CIF text and extracts loops and tokens for syntax highlighting

use crate::tokenizer::{is_block_keyword, is_data_name, is_loop_keyword, special_split};
use crate::{DataLine, Item, LoopBlock, ParseResult, Token, ValueRange};
use regex::Regex;

/// Parse CIF text and return loops and tokens
pub fn parse_cif_internal(text: &str) -> ParseResult {
    let mut loops: Vec<LoopBlock> = Vec::new();
    let mut tokens: Vec<Token> = Vec::new();
    let mut current_loop: Option<LoopBlock> = None;
    let mut multi_line_mode = false;

    // For non-loop items color rotation
    let mut last_category = String::new();
    let mut category_item_count: usize = 0;

    // Regex for category.field pattern
    let category_field_re = Regex::new(r"^(_[A-Za-z0-9_]+)\.([A-Za-z0-9_\[\]]+)$").unwrap();
    let leading_category_re =
        Regex::new(r"^(\s*)(_[A-Za-z0-9_]+)\.([A-Za-z0-9_\[\]]+)").unwrap();

    let lines: Vec<&str> = text.lines().collect();

    for (line_num, line_text) in lines.iter().enumerate() {
        let first_char = line_text.chars().next();

        // Skip comment lines
        if first_char == Some('#') {
            if let Some(ref mut current) = current_loop {
                if !current.items.is_empty() && current.names_defined {
                    loops.push(current.clone());
                    current_loop = None;
                }
            }
            // rainbow10 for comments
            tokens.push(Token {
                line: line_num,
                start: 0,
                length: line_text.len(),
                token_type: 10,
                item_name: None,
            });
            continue;
        }

        // Handle multi-line strings (semicolon delimiter)
        if first_char == Some(';') {
            if multi_line_mode {
                // End of multi-line string
                multi_line_mode = false;
                let token_type = if let Some(ref mut current) = current_loop {
                    let field_count = current.items.len().max(1);
                    let col_index = if current.is_in_loop_block {
                        current.processed_value_count % field_count
                    } else {
                        0
                    };
                    let tt = if current.is_in_loop_block {
                        1 + (col_index % 8) as u8
                    } else {
                        2
                    };

                    current.data_lines.push(DataLine {
                        line: line_num,
                        value_ranges: vec![ValueRange {
                            start: 0,
                            length: line_text.len(),
                            column_index: col_index,
                        }],
                    });
                    current.processed_value_count += 1;
                    tt
                } else {
                    2
                };

                let item_name = get_current_item_name(&current_loop);
                tokens.push(Token {
                    line: line_num,
                    start: 0,
                    length: line_text.len(),
                    token_type,
                    item_name,
                });

                if let Some(ref mut current) = current_loop {
                    if !current.items.is_empty() {
                        current.names_defined = true;
                    }
                }
            } else {
                // Start of multi-line string
                multi_line_mode = true;
                let token_type = if let Some(ref mut current) = current_loop {
                    let field_count = current.items.len().max(1);
                    let col_index = if current.is_in_loop_block {
                        current.processed_value_count % field_count
                    } else {
                        0
                    };
                    let tt = if current.is_in_loop_block {
                        1 + (col_index % 8) as u8
                    } else {
                        2
                    };

                    current.data_lines.push(DataLine {
                        line: line_num,
                        value_ranges: vec![ValueRange {
                            start: 0,
                            length: line_text.len(),
                            column_index: col_index,
                        }],
                    });
                    tt
                } else {
                    2
                };

                let item_name = get_current_item_name(&current_loop);
                tokens.push(Token {
                    line: line_num,
                    start: 0,
                    length: line_text.len(),
                    token_type,
                    item_name,
                });
            }
            continue;
        }

        // Inside multi-line string
        if multi_line_mode {
            if let Some(ref mut current) = current_loop {
                let field_count = current.items.len().max(1);
                let col_index = if current.is_in_loop_block {
                    current.processed_value_count % field_count
                } else {
                    0
                };

                current.data_lines.push(DataLine {
                    line: line_num,
                    value_ranges: vec![ValueRange {
                        start: 0,
                        length: line_text.len(),
                        column_index: col_index,
                    }],
                });

                if !line_text.is_empty() {
                    let token_type = if current.is_in_loop_block {
                        1 + (col_index % 8) as u8
                    } else {
                        2
                    };
                    let item_name = if !current.items.is_empty() {
                        let field_info = &current.items[col_index.min(current.items.len() - 1)];
                        Some(format!("{}.{}", current.category_name, field_info.name))
                    } else {
                        None
                    };
                    tokens.push(Token {
                        line: line_num,
                        start: 0,
                        length: line_text.len(),
                        token_type,
                        item_name,
                    });
                }
            }
            continue;
        }

        // Skip empty lines
        let trimmed = line_text.trim();
        if trimmed.is_empty() {
            if let Some(ref mut current) = current_loop {
                if !current.items.is_empty() && !current.names_defined {
                    current.names_defined = true;
                }
            }
            continue;
        }

        let line_tokens = special_split(trimmed);
        if line_tokens.is_empty() {
            continue;
        }

        // Check for block keyword (data_, save_, global_)
        if is_block_keyword(&line_tokens[0].0, line_tokens[0].1) {
            if let Some(ref current) = current_loop {
                if !current.items.is_empty() {
                    loops.push(current.clone());
                }
            }
            current_loop = None;

            // rainbow8 for section/heading
            if let Some(idx) = line_text.find(&line_tokens[0].0) {
                tokens.push(Token {
                    line: line_num,
                    start: idx,
                    length: line_tokens[0].0.len(),
                    token_type: 8,
                    item_name: None,
                });
            }
            continue;
        }

        // Check for loop keyword
        if is_loop_keyword(&line_tokens[0].0, line_tokens[0].1) {
            if let Some(ref current) = current_loop {
                if !current.items.is_empty() {
                    loops.push(current.clone());
                }
            }
            current_loop = Some(LoopBlock {
                start_line: line_num,
                category_name: String::new(),
                items: Vec::new(),
                names_defined: false,
                is_in_loop_block: true,
                processed_value_count: 0,
                data_lines: Vec::new(),
            });

            // Reset category tracking
            last_category.clear();
            category_item_count = 0;

            // rainbow6 for control
            if let Some(idx) = line_text.find(&line_tokens[0].0) {
                tokens.push(Token {
                    line: line_num,
                    start: idx,
                    length: line_tokens[0].0.len(),
                    token_type: 6,
                    item_name: None,
                });
            }
            continue;
        }

        // Check for data name (_category.field)
        if is_data_name(&line_tokens[0].0, line_tokens[0].1) {
            let data_name = &line_tokens[0].0;

            if let Some(caps) = category_field_re.captures(data_name) {
                let category_name = caps.get(1).unwrap().as_str().to_string();
                let name = caps.get(2).unwrap().as_str().to_string();

                if let Some(lead_caps) = leading_category_re.captures(line_text) {
                    let leading_spaces = lead_caps.get(1).map_or(0, |m| m.as_str().len());
                    let field_start = leading_spaces + category_name.len() + 1;
                    let field_length = name.len();

                    // Handle category transitions for non-loop blocks
                    if let Some(ref current) = current_loop {
                        if current.names_defined && !current.items.is_empty() {
                            if current.category_name != category_name
                                || (current.category_name == category_name
                                    && current.items.len() == 1)
                            {
                                loops.push(current.clone());
                                current_loop = None;
                            }
                        }
                    }

                    if current_loop.is_none() {
                        current_loop = Some(LoopBlock {
                            start_line: line_num,
                            category_name: category_name.clone(),
                            items: Vec::new(),
                            names_defined: true,
                            is_in_loop_block: false,
                            processed_value_count: 0,
                            data_lines: Vec::new(),
                        });
                    }

                    let current = current_loop.as_mut().unwrap();

                    if current.category_name.is_empty() {
                        current.category_name = category_name.clone();
                    }

                    if current.category_name != category_name {
                        if !current.items.is_empty() {
                            loops.push(current.clone());
                        }
                        *current = LoopBlock {
                            start_line: line_num,
                            category_name: category_name.clone(),
                            items: Vec::new(),
                            names_defined: !current.is_in_loop_block,
                            is_in_loop_block: current.is_in_loop_block,
                            processed_value_count: 0,
                            data_lines: Vec::new(),
                        };
                    }

                    current.items.push(Item {
                        line: line_num,
                        start: field_start,
                        length: field_length,
                        name: name.clone(),
                    });

                    // Track category item count for non-loop color rotation
                    let color_base_index = if !current.is_in_loop_block {
                        if category_name != last_category {
                            category_item_count = 0;
                            last_category = category_name.clone();
                        }
                        let idx = category_item_count;
                        category_item_count += 1;
                        idx
                    } else {
                        0
                    };

                    // Generate tokens for category and field name
                    let field_index = current.items.len() - 1;
                    let token_type_index = if current.is_in_loop_block {
                        2 + (field_index % 7) as u8
                    } else {
                        2 + (color_base_index % 7) as u8
                    };

                    // Category part -> rainbow1
                    tokens.push(Token {
                        line: line_num,
                        start: leading_spaces,
                        length: category_name.len(),
                        token_type: 1,
                        item_name: None,
                    });

                    // Dot + Field Name -> rotating color
                    tokens.push(Token {
                        line: line_num,
                        start: leading_spaces + category_name.len(),
                        length: 1 + field_length,
                        token_type: token_type_index,
                        item_name: None,
                    });

                    // Process values on the same line
                    if line_tokens.len() > 1 {
                        let token_text = &line_tokens[1].0;
                        let search_start = lead_caps.get(0).map_or(0, |m| m.end());
                        if let Some(rel_idx) = line_text[search_start..].find(token_text.as_str()) {
                            let idx = search_start + rel_idx;
                            let column_index = color_base_index;

                            current.data_lines.push(DataLine {
                                line: line_num,
                                value_ranges: vec![ValueRange {
                                    start: idx,
                                    length: token_text.len(),
                                    column_index,
                                }],
                            });
                            current.processed_value_count += 1;

                            let val_token_type = 2 + (column_index % 7) as u8;
                            let item_name = Some(format!(
                                "{}.{}",
                                current.category_name,
                                current.items[field_index].name
                            ));
                            tokens.push(Token {
                                line: line_num,
                                start: idx,
                                length: token_text.len(),
                                token_type: val_token_type,
                                item_name,
                            });
                        }
                    }
                }
            } else {
                // Data name without category.field pattern
                if let Some(idx) = line_text.find(data_name.as_str()) {
                    tokens.push(Token {
                        line: line_num,
                        start: idx,
                        length: data_name.len(),
                        token_type: 1,
                        item_name: None,
                    });
                }
            }
        } else if let Some(ref mut current) = current_loop {
            // Value lines in loop block
            if !current.items.is_empty() {
                if !current.names_defined {
                    current.names_defined = true;
                }

                let mut value_ranges: Vec<ValueRange> = Vec::new();
                let field_count = current.items.len().max(1);
                let mut search_start = 0usize;

                for (col, (token_text, _)) in line_tokens.iter().enumerate() {
                    if let Some(rel_idx) = line_text[search_start..].find(token_text.as_str()) {
                        let idx = search_start + rel_idx;
                        let current_total = current.processed_value_count + col;
                        let effective_col_index = current_total % field_count;

                        value_ranges.push(ValueRange {
                            start: idx,
                            length: token_text.len(),
                            column_index: effective_col_index,
                        });

                        let token_type = 2 + (effective_col_index % 7) as u8;
                        let item_name = if effective_col_index < current.items.len() {
                            Some(format!(
                                "{}.{}",
                                current.category_name,
                                current.items[effective_col_index].name
                            ))
                        } else {
                            None
                        };

                        tokens.push(Token {
                            line: line_num,
                            start: idx,
                            length: token_text.len(),
                            token_type,
                            item_name,
                        });

                        search_start = idx + token_text.len();
                    }
                }

                if !value_ranges.is_empty() {
                    current.data_lines.push(DataLine {
                        line: line_num,
                        value_ranges,
                    });
                    current.processed_value_count += line_tokens.len();
                }
            }
        }
    }

    // Push final loop if exists
    if let Some(current) = current_loop {
        if !current.items.is_empty() {
            loops.push(current);
        }
    }

    ParseResult { loops, tokens }
}

/// Helper to get current item name from loop context
fn get_current_item_name(current_loop: &Option<LoopBlock>) -> Option<String> {
    if let Some(ref current) = current_loop {
        if !current.items.is_empty() {
            let field_count = current.items.len();
            let col_index = if current.is_in_loop_block {
                current.processed_value_count % field_count
            } else {
                0
            };
            if col_index < current.items.len() {
                return Some(format!(
                    "{}.{}",
                    current.category_name, current.items[col_index].name
                ));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_parse() {
        let cif = r#"
data_test
_entry.id TEST
loop_
_atom_site.id
_atom_site.type_symbol
1 C
2 N
"#;
        let result = parse_cif_internal(cif);
        assert!(!result.loops.is_empty());
        assert!(!result.tokens.is_empty());
    }
}
