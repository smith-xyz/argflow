# Argflow

Argument flow analyzer - traces where function arguments come from across multi-language codebases. It identifies API calls matching configurable presets and resolves parameter values through static analysis.

## Building

```bash
cargo build --release
```

The binary will be at `target/release/argflow`.

## Usage

### Basic Usage

Analyze a single file with the crypto preset:

```bash
argflow --preset crypto --path path/to/file.go --language go
```

Analyze a directory:

```bash
argflow --preset crypto --path path/to/project --language go
```

### Presets

Argflow uses presets to define which APIs to analyze. The `crypto` preset is bundled and used by default.

```bash
# Use bundled crypto preset (default)
argflow --path ./project --language go

# Explicitly specify preset
argflow --preset crypto --path ./project --language go

# Use custom rules file
argflow --rules ./my-rules.json --path ./project --language go
```

### Options

- `--path <PATH>` - Path to file or directory to analyze (required)
- `--preset <PRESET>` - Preset to use (e.g., crypto). Can be specified multiple times.
- `--rules <FILE>` - Custom rules file (JSON format)
- `--language <LANGUAGE>` - Language (go, python, rust, javascript, typescript). Auto-detected for single files.
- `--include-deps` - Include dependencies (vendor/, node_modules/, etc.)
- `-O, --output-file <FILE>` - Output file path (prints to stdout if not specified)
- `-f, --format <FORMAT>` - Output format: json or cbom (default: json)
- `-v, --verbose` - Increase verbosity (-v info, -vv debug, -vvv trace)
- `-q, --quiet` - Suppress all output except errors

### Examples

Analyze a Go project with dependencies:

```bash
argflow --preset crypto --path ./my-project --language go --include-deps
```

Analyze a Python file:

```bash
argflow --preset crypto --path src/crypto.py --language python
```

Save output to file:

```bash
argflow --preset crypto --path ./project --language go -O findings.json
```

## Output Format

The tool outputs JSON with the following structure:

```json
{
  "files_scanned": 217,
  "total_calls": 518,
  "total_configs": 36,
  "findings": [
    {
      "file": "/path/to/file.go",
      "line": 11,
      "column": 10,
      "function": "Sum",
      "package": "md5",
      "import_path": "crypto/md5",
      "full_name": "md5.Sum",
      "algorithm": "MD5",
      "finding_type": "hash",
      "operation": "hash",
      "primitive": "hash",
      "parameters": {
        "data": {
          "value": null,
          "source": "function_parameter"
        }
      },
      "raw_text": "md5.Sum([]byte(infraID))"
    }
  ],
  "configs": [
    {
      "file": "/path/to/config.go",
      "line": 42,
      "column": 5,
      "struct_type": "TLSConfig",
      "full_type": "crypto/tls.Config",
      "package": "tls",
      "import_path": "crypto/tls",
      "fields": {
        "MinVersion": 771
      },
      "raw_text": "&tls.Config{MinVersion: tls.VersionTLS12}"
    }
  ]
}
```

### Output Fields

- `files_scanned` - Number of files analyzed
- `total_calls` - Total API calls found matching the preset
- `total_configs` - Total configuration structs found
- `findings` - Array of API call findings
- `configs` - Array of configuration struct findings

### Parameter Resolution

Parameters can be:

- Resolved: Direct value extracted (e.g., `2048`, `"SHA-256"`)
- Partial: Expression extracted (e.g., `"BASE + 1000"` with `source: "partial_expression"`)
- Unresolved: Source identified but value unknown (e.g., `source: "function_parameter"`)

## Supported Languages

- Go
- Python
- Rust
- JavaScript/TypeScript

## How It Works

Argflow uses Tree-sitter to parse source code into ASTs, then applies resolution strategies to trace argument values:

1. **Literal values** - Direct constants
2. **Variable resolution** - Finds variable declarations and constants
3. **Function calls** - Traces return values
4. **Binary expressions** - Evaluates arithmetic operations
5. **Field access** - Resolves struct/object fields
6. **Array/index access** - Resolves array and map lookups

The tool uses preset-defined API mappings to identify which function calls to analyze and how to classify them.

## Presets

See `presets/README.md` for information on bundled presets and creating custom presets.

### Bundled Presets

| Preset   | Description                                                       |
| -------- | ----------------------------------------------------------------- |
| `crypto` | Cryptographic APIs - key derivation, encryption, hashing, signing |

## License

MIT
