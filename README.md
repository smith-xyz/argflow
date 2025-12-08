# Crypto Extractor

A static analysis tool that extracts cryptographic parameters from codebases. It identifies crypto API calls and attempts to resolve parameter values through static analysis.

## Building

```bash
cargo build --release
```

The binary will be at `target/release/crypto-extractor`.

## Usage

### Basic Usage

Analyze a single file:

```bash
crypto-extractor --path path/to/file.go --language go
```

Analyze a directory:

```bash
crypto-extractor --path path/to/project --language go
```

### Options

- `--path <PATH>` - Path to file or directory to analyze (required)
- `--language <LANGUAGE>` - Language (go, python, rust, javascript, typescript). Auto-detected for single files.
- `--include-deps` - Include dependencies (vendor/, node_modules/, etc.)
- `--output <FORMAT>` - Output format: json or cbom (default: json)
- `-v, --verbose` - Increase verbosity (-v info, -vv debug, -vvv trace)
- `-q, --quiet` - Suppress all output except errors

### Examples

Analyze a Go project with dependencies:

```bash
crypto-extractor --path ./my-project --language go --include-deps
```

Analyze a Python file:

```bash
crypto-extractor --path src/crypto.py --language python
```

Save output to file:

```bash
crypto-extractor --path ./project --language go > findings.json
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
      "arguments": [
        {
          "index": 0,
          "resolved": false,
          "value": {
            "unresolved": "not_implemented"
          }
        }
      ],
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
      "fields": [
        {
          "field_name": "MinVersion",
          "resolved": true,
          "value": 771,
          "classification_key": "tls_version"
        }
      ],
      "raw_text": "&tls.Config{MinVersion: tls.VersionTLS12}"
    }
  ]
}
```

### Output Fields

- `files_scanned` - Number of files analyzed
- `total_calls` - Total cryptographic function calls found
- `total_configs` - Total crypto configuration structs found
- `findings` - Array of cryptographic function call findings
- `configs` - Array of crypto configuration struct findings

### Finding Fields

- `file` - Full path to the source file
- `line` - Line number where the call occurs
- `column` - Column number where the call occurs
- `function` - Function name
- `package` - Package name
- `import_path` - Full import path
- `algorithm` - Cryptographic algorithm identified
- `finding_type` - Type of finding (hash, cipher, kdf, etc.)
- `operation` - Operation type (hash, encrypt, decrypt, etc.)
- `primitive` - Cryptographic primitive
- `arguments` - Array of function arguments with resolution status
- `raw_text` - Original source code text

### Argument Resolution

Arguments can be:

- Resolved: `{"resolved": true, "value": 2048}` - Actual value extracted
- Unresolved: `{"resolved": false, "value": {"unresolved": "reason"}}` - Could not resolve
- Partial: `{"resolved": false, "value": {"expression": "BASE + 1000", "partial": true}}` - Expression extracted

## Supported Languages

- Go
- Python
- Rust
- JavaScript/TypeScript

## How It Works

The tool uses Tree-sitter to parse source code into ASTs, then applies resolution strategies to extract cryptographic parameters:

1. Literal values - Direct constants
2. Variable resolution - Finds variable declarations
3. Function calls - Traces return values
4. Binary expressions - Evaluates arithmetic operations
5. Field access - Resolves struct/object fields
6. Array/index access - Resolves array and map lookups

The tool uses API mappings to identify cryptographic functions and classify them by algorithm and operation type.
