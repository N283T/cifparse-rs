mod parser;
mod tokenizer;
mod wasm;

use serde::{Deserialize, Serialize};

// Re-export WASM bindings
pub use wasm::CifParser;

/// Token information for syntax highlighting
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Token {
    pub line: usize,
    pub start: usize,
    pub length: usize,
    pub token_type: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_name: Option<String>,
}

/// Item (field) information within a category
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Item {
    pub line: usize,
    pub start: usize,
    pub length: usize,
    pub name: String,
}

/// Value range information
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ValueRange {
    pub start: usize,
    pub length: usize,
    pub column_index: usize,
}

/// Data line information
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DataLine {
    pub line: usize,
    pub value_ranges: Vec<ValueRange>,
}

/// Loop block information
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LoopBlock {
    pub start_line: usize,
    pub category_name: String,
    pub items: Vec<Item>,
    pub names_defined: bool,
    pub is_in_loop_block: bool,
    pub processed_value_count: usize,
    pub data_lines: Vec<DataLine>,
}

/// Parse result containing loops and tokens
#[derive(Serialize, Deserialize, Debug)]
pub struct ParseResult {
    pub loops: Vec<LoopBlock>,
    pub tokens: Vec<Token>,
}

/// Parse CIF text and return result
pub fn parse(text: &str) -> ParseResult {
    parser::parse_cif_internal(text)
}

/// Parse CIF text and return as JSON string
pub fn parse_to_json(text: &str) -> String {
    let result = parser::parse_cif_internal(text);
    serde_json::to_string(&result).unwrap_or_default()
}
