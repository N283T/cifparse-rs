//! WASM bindings for CIF parser

use crate::parse;
use wasm_bindgen::prelude::*;

/// CIF Parser for WASM
#[wasm_bindgen]
pub struct CifParser;

#[wasm_bindgen]
impl CifParser {
    /// Create a new CIF parser instance
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        CifParser
    }

    /// Parse CIF text and return JSON result
    #[wasm_bindgen]
    pub fn parse(&self, text: &str) -> JsValue {
        let result = parse(text);
        serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
    }

    /// Parse CIF text and return only tokens (for syntax highlighting)
    #[wasm_bindgen]
    pub fn parse_tokens(&self, text: &str) -> JsValue {
        let result = parse(text);
        serde_wasm_bindgen::to_value(&result.tokens).unwrap_or(JsValue::NULL)
    }

    /// Parse CIF text and return only loops (for structure analysis)
    #[wasm_bindgen]
    pub fn parse_loops(&self, text: &str) -> JsValue {
        let result = parse(text);
        serde_wasm_bindgen::to_value(&result.loops).unwrap_or(JsValue::NULL)
    }
}

impl Default for CifParser {
    fn default() -> Self {
        Self::new()
    }
}
