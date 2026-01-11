# cifparse-rs

A CIF/mmCIF format parser written in Rust, compiled to WebAssembly (WASM) for use in Chrome and VS Code extensions.

## Build

```bash
# WASM build
wasm-pack build --target web --release

# Native build
cargo build --release

# Test
cargo test
```

## Usage (JavaScript/TypeScript)

```javascript
import init, { CifParser } from './pkg/cif_parser.js';

await init();
const parser = new CifParser();

const result = parser.parse(cifText);
// result = { loops: [...], tokens: [...] }

// Tokens only (for syntax highlighting)
const tokens = parser.parse_tokens(cifText);

// Loops only (for structure analysis)
const loops = parser.parse_loops(cifText);
```

## Data Structures

### ParseResult
```typescript
interface ParseResult {
  loops: LoopBlock[];
  tokens: Token[];
}
```

### LoopBlock
```typescript
interface LoopBlock {
  start_line: number;
  category_name: string;        // e.g., "_atom_site"
  items: Item[];                // Item definitions in this category
  names_defined: boolean;
  is_in_loop_block: boolean;
  processed_value_count: number;
  data_lines: DataLine[];
}
```

### Item
```typescript
interface Item {
  line: number;
  start: number;
  length: number;
  name: string;    // e.g., "id", "type_symbol"
}
```

### Token
```typescript
interface Token {
  line: number;
  start: number;
  length: number;
  token_type: number;     // rainbow color index
  item_name?: string;     // e.g., "_atom_site.id"
}
```

## Supported Formats

- `data_` / `save_` / `global_` blocks
- `loop_` constructs
- Semicolon-delimited multi-line strings (`;` ... `;`)
- Single/double quoted strings
- `#` comments
- `_category.item` data names

## License

MIT
