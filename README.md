# PDF Diff Tool

A Rust-based tool for comparing PDF documents by converting them to images and generating visual diffs to highlight changes between documents.

## Features

- **PDF to Image Conversion**: Converts PDF pages to high-resolution images using PDFium
- **Visual Diff Generation**: Creates difference images highlighting changes between PDF versions
- **Smart Cropping**: Automatically crops images to content areas, removing whitespace
- **Batch Processing**: Processes multiple pages and generates individual diff images
- **High Quality Output**: Configurable DPI settings for crisp, detailed output images

## Prerequisites

- Rust (latest stable version)
- PDFium library for macOS ARM64 (included in `pdfium-mac-arm64/`)

## Installation

1. Clone the repository:
```bash
git clone <your-repo-url>
cd pdf_diff
```

2. Build the project:
```bash
cargo build --release
```

## Usage

### Command Line Usage

```bash
# Basic usage
cargo run -- --old path/to/old.pdf --new path/to/new.pdf

# With custom output directory
cargo run -- --old old.pdf --new new.pdf --output-dir my_output

# High quality rendering
cargo run -- --old old.pdf --new new.pdf --dpi 600

# More sensitive diff detection
cargo run -- --old old.pdf --new new.pdf --sensitivity 0.05

# Verbose output
cargo run -- --old old.pdf --new new.pdf --verbose

# All options combined
cargo run -- --old old.pdf --new new.pdf --output-dir results --dpi 600 --sensitivity 0.08 --verbose

# Or if you've built a release binary:
./target/release/pdf_diff --old old.pdf --new new.pdf
```

### Command Line Options

- `--old, -o`: Path to the old PDF file (required)
- `--new, -n`: Path to the new PDF file (required)
- `--output-dir, -d`: Directory to save diff images (default: "output")
- `--dpi`: DPI for PDF rendering (default: 300, higher = better quality)
- `--sensitivity`: Diff sensitivity threshold 0.0-1.0 (default: 0.12, lower = more sensitive)
- `--verbose, -v`: Enable verbose output
- `--help, -h`: Show help message

### Legacy Usage (Hardcoded Paths)

Place your PDF files in the `samples/` directory and update the file paths in `src/main.rs`:

```rust
let path_old = path::Path::new("./samples/sample_3.pdf");
let path_new = path::Path::new("./samples/sample_4.pdf");
```

Run the tool:
```bash
cargo run
```

### Library Usage

You can also use this as a library in your own Rust projects:

```rust
use pdf_diff::pdf::{create_pdfium, load_pdf_documents, create_images_from_pdf};
use pdf_diff::image_utils::{diff_images, save_images};

// Create PDFium instance
let pdfium = create_pdfium()?;

// Load PDF documents
let (old_doc, new_doc) = load_pdf_documents(&pdfium, old_path, new_path)?;

// Convert to images with custom DPI
let images = create_images_from_pdf(&old_doc, &new_doc, 600.0)?;

// Generate diffs with custom sensitivity
let diff_images = diff_images(images, 0.08)?;

// Save results
save_images(diff_images, "output")?;
```

## Configuration

### Command Line Configuration (Recommended)

All settings can be configured via command-line arguments:

- `--dpi 600`: Higher DPI for better quality (default: 300)
- `--sensitivity 0.05`: Lower values for more sensitive diff detection (default: 0.12)
- `--output-dir custom_output`: Change output directory (default: "output")

### Code Configuration (For Library Use)

When using as a library, you can configure these settings programmatically:

```rust
// Custom DPI for image quality
let images = create_images_from_pdf(&old_doc, &new_doc, 600.0)?;

// Custom sensitivity for diff detection
let diff_images = diff_images(images, 0.05)?;
```

### Advanced Configuration

For advanced users who want to modify the source code:

#### Cropping Tolerance

Adjust the white pixel tolerance in `src/lib/image_utils.rs`:

```rust
let tolerance = 10u8; // Higher values = more aggressive cropping
```

## Project Structure

```
pdf_diff/
├── src/
│   ├── main.rs              # Main application entry point
│   └── lib/
│       ├── mod.rs           # Library module declarations
│       ├── pdf.rs           # PDF processing and rendering
│       └── image_utils.rs   # Image manipulation and diff utilities
├── samples/                 # Sample PDF files for testing
├── output/                  # Generated diff images
├── pdfium-mac-arm64/        # PDFium library files
├── Cargo.toml              # Rust dependencies and configuration
└── README.md               # This file
```

## Dependencies

- `clap` - Command-line argument parsing
- `pdfium-render` - PDF rendering using PDFium
- `image` - Image processing and manipulation
- `diff_img` - Image diffing algorithms
- `anyhow` - Error handling

## Testing

Run the test suite:

```bash
cargo test
```

The tests cover:
- PDF loading and validation
- Image rendering from PDF pages
- Image cropping and manipulation
- Diff generation
- File I/O operations

## Output

The tool generates:

1. **Diff Images**: Visual representations of changes between PDF versions
2. **Cropped Content**: Images are automatically cropped to remove excess whitespace
3. **High Resolution**: Images rendered at configurable DPI for quality output
4. **PNG Format**: Lossless compression for accurate diff visualization

## Error Handling

The tool provides detailed error messages for common issues:
- Missing PDF files
- Corrupted PDF documents
- PDFium library loading issues
- File permission problems
- Invalid image dimensions

## Limitations

- Currently optimized for macOS ARM64 architecture
- Requires PDFium library to be present
- Memory usage scales with PDF size and DPI settings
- Processing time increases with higher DPI and larger documents

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

MIT License - see LICENSE file for details.

## Troubleshooting

### PDFium Library Issues

If you encounter `libpdfium.dylib` loading errors:

1. Ensure the library path is correct in your code
2. Verify the library exists at `./pdfium-mac-arm64/lib/libpdfium.dylib`
3. Check that you're using the correct architecture version

### Low Quality Output

- Increase the DPI setting in the configuration
- Ensure source PDFs are vector-based, not raster images
- Check that cropping isn't removing important content

### Performance Issues

- Reduce DPI for faster processing
- Process PDFs in smaller batches
- Consider using release builds (`cargo build --release`)

## Examples

See the `samples/` directory for example PDF files that demonstrate the tool's capabilities.