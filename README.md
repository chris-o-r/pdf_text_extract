# PDF Text Extract

A high-performance Rust tool for extracting text from PDF documents with precise bounding box information. This tool can analyze PDF documents, extract text with spatial coordinates, group text into paragraphs, and generate visual debugging output.

## Features

- **Text Extraction**: Extract text from PDF documents with precise positioning information
- **Bounding Box Detection**: Get exact coordinates for each text element
- **Flexible Chunking**: Choose between paragraph-level or line-level text grouping
- **Debug Visualization**: Generate images with bounding boxes overlaid for debugging
- **JSON Output**: Export structured text data as JSON
- **Performance Benchmarking**: Built-in timing and performance metrics

## Installation

### Prerequisites

- Rust 1.75+ (2024 edition)
- PDFium binaries (see setup instructions below)

### PDFium Binary Setup

This project requires PDFium binaries to function. You need to:

1. Download the appropriate PDFium binaries for your platform from [PDFium releases](https://github.com/paulocoutinhox/pdfium-lib/releases)
2. Extract the binaries to a `pdfium-<platform>` directory in the project root
3. Ensure the directory structure matches:
   ```
   pdfium-<platform>/
   ├── include/        # Header files
   ├── lib/           # Library files
   └── LICENSE        # PDFium license
   ```

**Note**: The current code expects `pdfium-mac-arm64/` for macOS ARM64. Adjust the path in your code for other platforms.

### Building from Source

```bash
git clone <your-repo-url>
cd pdf_text_extract
# Set up PDFium binaries (see above)
cargo build --release
```

## Usage

### Basic Text Extraction

Extract text and print to stdout:
```bash
cargo run -- --pdf samples/document.pdf
```

### Save Output to JSON

Extract text and save to `output/output.json`:
```bash
cargo run -- --pdf samples/document.pdf --output
```

### Debug Mode with Bounding Boxes

Generate visual debugging images with bounding boxes:
```bash
cargo run -- --pdf samples/document.pdf --debug
```

### Chunking Strategies

Choose how text is grouped (paragraph or line level):
```bash
# Group text into paragraphs (default)
cargo run -- --pdf samples/document.pdf --chunking paragraph

# Extract individual lines
cargo run -- --pdf samples/document.pdf --chunking line
```

### Benchmark Performance

Run with performance timing:
```bash
cargo run -- --pdf samples/document.pdf --benchmark
```

### Combined Options

```bash
cargo run -- --pdf samples/document.pdf --output --debug --benchmark --chunking paragraph
```

## Command Line Options

| Option | Short | Description | Default |
|--------|--------|-------------|---------|
| `--pdf` | `-p` | Path to the PDF file | Required |
| `--chunking` | `-c` | Text chunking strategy (`paragraph` or `line`) | `paragraph` |
| `--output` | `-o` | Save output as JSON to `output/output.json` | `false` |
| `--debug` | `-d` | Generate debug images with bounding boxes | `false` |
| `--benchmark` | `-b` | Show performance timing information | `false` |

## Output Format

The tool outputs structured JSON with the following format:

```json
[
  {
    "page_index": 0,
    "text": "This is a paragraph of text extracted from the PDF.",
    "bounding_box": {
      "x": 72.0,
      "y": 100.5,
      "width": 450.2,
      "height": 14.4
    }
  }
]
```

### Field Descriptions

- `page_index`: Zero-based page number
- `text`: The extracted text content (grouped into paragraphs)
- `bounding_box`: Spatial coordinates in PDF points (72 DPI)
  - `x`, `y`: Top-left corner coordinates
  - `width`, `height`: Dimensions of the text block

## Architecture

The project is structured as follows:

```
src/
├── main.rs              # CLI interface and main application logic
└── lib/
    ├── mod.rs           # Library module declarations
    ├── models.rs        # Data structures (PdfPageText, BoundingBox)
    ├── pdf.rs           # PDF processing and text extraction
    ├── chunker.rs       # Text grouping and paragraph detection
    └── image_utils.rs   # Image processing and debug visualization
```

### Key Components

- **PDF Processing**: Uses PDFium for robust PDF parsing and text extraction
- **Text Chunking**: Configurable algorithms to group text elements (paragraph or line level)
- **Bounding Box Calculation**: Precise spatial coordinate calculation for text positioning
- **Debug Visualization**: Overlay bounding boxes on rendered PDF pages for verification

## Dependencies

- `pdfium-render`: PDF processing and rendering
- `image`: Image manipulation for debug output
- `clap`: Command-line argument parsing
- `serde`: JSON serialization/deserialization
- `anyhow`: Error handling

## Performance

The tool includes built-in benchmarking. Typical performance on modern hardware:

- **Processing Speed**: ~50-200ms per page (depends on text density and chunking strategy)
- **Memory Usage**: Scales with document size and chunking strategy
- **Output**: Processing time, pages per second, and text chunk count metrics

## Examples

The `samples/` directory contains example PDF files for testing:

```bash
# Process a sample document
cargo run -- --pdf samples/rpa0038.pdf --output --benchmark

# Generate debug visualization
cargo run -- --pdf samples/1281082.pdf --debug --chunking paragraph
```

## Troubleshooting

### Common Issues

1. **File Not Found**: Ensure the PDF path is correct and the file exists
2. **PDFium Initialization Failed**: Check that PDFium binaries are properly included
3. **Memory Issues**: Try reducing DPI for large documents
4. **Invalid Rectangles**: Some PDFs may have malformed text coordinates

### Debug Mode

Use `--debug` to generate visual output that shows:
- Extracted text bounding boxes as colored rectangles
- Page-by-page processing information
- Coordinate transformation details


## Acknowledgments

- Built with [PDFium](https://pdfium.googlesource.com/pdfium/) for reliable PDF processing
- Uses Rust's ecosystem for high-performance text processing