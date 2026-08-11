//! `easyofd` CLI -- command-line tool for OFD document operations.
//!
//! Subcommands: `info`, `to-markdown`, `to-pdf`, `sign`, `verify`, `pages`.

use clap::{Parser, Subcommand};
use easyofd::{
    ConvertOptions, EasyOfd, ElectronicSeal, OfdSignatureBuilder, SignatureAlgorithm,
    read_signature,
};
use std::process::ExitCode;

// ─── CLI Definition ──────────────────────────────────────────────────────────

/// OFD document CLI -- read, convert, sign, and verify OFD files.
#[derive(Parser)]
#[command(name = "easyofd", version, about = "OFD document CLI")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Read an OFD file and print page count + extracted text.
    Info {
        /// Path to the OFD file.
        path: String,
    },
    /// Convert OFD to Markdown.
    ToMarkdown {
        /// Path to the input OFD file.
        ofd: String,
        /// Output file path (defaults to stdout).
        output: Option<String>,
    },
    /// Convert OFD to PDF.
    ToPdf {
        /// Path to the input OFD file.
        ofd: String,
        /// Path for the output PDF file.
        pdf: String,
    },
    /// Digitally sign an OFD document (SM2WithSM3, demo key).
    Sign {
        /// Path to the input OFD file.
        input: String,
        /// Path for the signed output OFD file.
        output: String,
        /// Path to PEM-encoded SM2 private key (reserved for future use).
        #[arg(long)]
        key: Option<String>,
        /// Path to PEM-encoded X.509 certificate (reserved for future use).
        #[arg(long)]
        cert: Option<String>,
    },
    /// Verify the digital signature of an OFD document.
    Verify {
        /// Path to the signed OFD file.
        path: String,
    },
    /// List all pages in an OFD document with content summary.
    Pages {
        /// Path to the OFD file.
        path: String,
    },
}

// ─── Main ────────────────────────────────────────────────────────────────────

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.cmd {
        Cmd::Info { path } => cmd_info(&path),
        Cmd::ToMarkdown { ofd, output } => cmd_to_markdown(&ofd, output.as_deref()),
        Cmd::ToPdf { ofd, pdf } => cmd_to_pdf(&ofd, &pdf),
        Cmd::Sign {
            input,
            output,
            key,
            cert,
        } => cmd_sign(&input, &output, key.as_deref(), cert.as_deref()),
        Cmd::Verify { path } => cmd_verify(&path),
        Cmd::Pages { path } => cmd_pages(&path),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

// ─── Subcommand Implementations ──────────────────────────────────────────────

fn cmd_info(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = check_exists(path)?;
    let reader = EasyOfd::read(&path)?;
    println!("File    : {}", path.display());
    println!("Pages   : {}", reader.page_count());
    let texts = reader.extract_text();
    for (i, text) in texts.iter().enumerate() {
        let preview: String = text.chars().take(200).collect();
        if preview.is_empty() {
            println!("Page {} : (no text)", i + 1);
        } else {
            println!("Page {} : {}", i + 1, preview);
        }
    }
    Ok(())
}

fn cmd_to_markdown(ofd: &str, output: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let ofd = check_exists(ofd)?;
    let result = EasyOfd::to_markdown(&ofd).do_convert()?;
    match output {
        Some(out_path) => {
            std::fs::write(out_path, &result.markdown)?;
            println!("Written to {out_path}");
        }
        None => {
            println!("{}", result.markdown);
        }
    }
    println!(
        "(pages_converted={}, losses={})",
        result.report.pages_converted,
        result.report.losses.len()
    );
    Ok(())
}

fn cmd_to_pdf(ofd: &str, pdf: &str) -> Result<(), Box<dyn std::error::Error>> {
    let ofd = check_exists(ofd)?;
    let options = ConvertOptions::default();
    easyofd::ofd_to_pdf(&ofd, pdf, &options)?;
    println!("Converted: {} -> {pdf}", ofd.display());
    Ok(())
}

fn cmd_sign(
    input: &str,
    output: &str,
    _key: Option<&str>,
    _cert: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let input = check_exists(input)?;

    let seal = ElectronicSeal {
        image_data: demo_seal_png(),
        name: "Demo Seal".to_string(),
        position: (0.0, 0.0),
        page: 1,
    };

    let signed = OfdSignatureBuilder::new(input.to_string_lossy().as_ref())
        .seal(seal)
        .algorithm(SignatureAlgorithm::Sm2WithSm3)
        .sign()?;

    let digest_preview = &signed.digest[..signed.digest.len().min(64)];
    println!("Signed: {} -> {output}", input.display());
    println!("Digest: {digest_preview}");
    signed.save(output)?;
    Ok(())
}

fn cmd_verify(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = check_exists(path)?;
    let result = read_signature(&path)?;
    println!("Algorithm  : {:?}", result.algorithm);
    let digest_preview = if result.digest.len() > 64 {
        &result.digest[..64]
    } else {
        &result.digest
    };
    println!("Digest     : {digest_preview}");
    let sig_preview_len = result.signature_value.len().min(64);
    println!(
        "Sig value  : {}...",
        &result.signature_value[..sig_preview_len]
    );
    if result.public_key.is_empty() {
        println!("Public key : (none)");
    } else {
        let pk_preview_len = result.public_key.len().min(64);
        println!("Public key : {}...", &result.public_key[..pk_preview_len]);
    }
    if result.reference_failures.is_empty() {
        println!("References : OK (all file hashes match)");
    } else {
        println!(
            "References : FAIL ({} mismatched)",
            result.reference_failures.len()
        );
        for f in &result.reference_failures {
            println!("  - {f}");
        }
    }
    Ok(())
}

fn cmd_pages(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = check_exists(path)?;
    let reader = EasyOfd::read(&path)?;
    let count = reader.page_count();
    println!("Total pages: {count}");
    for (i, page) in reader.pages().iter().enumerate() {
        let mut texts = 0;
        let mut images = 0;
        let mut paths = 0;
        for obj in &page.content {
            match obj {
                easyofd::ContentObject::Text(_) => texts += 1,
                easyofd::ContentObject::Image(_) => images += 1,
                easyofd::ContentObject::Path(_) => paths += 1,
            }
        }
        println!(
            "  Page {}: {:.1}x{:.1} mm | {} text, {} image, {} path",
            i + 1,
            page.width,
            page.height,
            texts,
            images,
            paths
        );
    }
    Ok(())
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Validate that a path exists and return it as a `PathBuf`.
fn check_exists(path: &str) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let p = std::path::PathBuf::from(path);
    if !p.exists() {
        return Err(format!("file not found: {path}").into());
    }
    Ok(p)
}

/// Generate a minimal 32x32 red PNG for demo seal.
fn demo_seal_png() -> Vec<u8> {
    let width = 32u32;
    let height = 32u32;
    let mut png = Vec::new();
    // PNG signature
    png.extend_from_slice(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
    // IHDR
    let ihdr_data = {
        let mut d = Vec::new();
        d.extend_from_slice(&width.to_be_bytes());
        d.extend_from_slice(&height.to_be_bytes());
        d.extend_from_slice(&[8, 2, 0, 0, 0]); // 8-bit RGB
        d
    };
    append_chunk(&mut png, *b"IHDR", &ihdr_data);
    // IDAT: uncompressed deflate with red pixels
    let row_bytes = (width as usize) * 3 + 1; // +1 for filter byte
    let mut row = Vec::with_capacity(row_bytes);
    row.push(0u8); // filter none
    for _ in 0..width {
        row.extend_from_slice(&[0xCC, 0x00, 0x00]); // red
    }
    let mut raw = Vec::with_capacity(row_bytes * height as usize);
    for _ in 0..height {
        raw.extend_from_slice(&row);
    }
    let compressed = deflate_raw(&raw);
    append_chunk(&mut png, *b"IDAT", &compressed);
    // IEND
    append_chunk(&mut png, *b"IEND", &[]);
    png
}

#[allow(clippy::cast_possible_truncation)]
fn append_chunk(png: &mut Vec<u8>, chunk_type: [u8; 4], data: &[u8]) {
    let len = u32::try_from(data.len()).unwrap_or(u32::MAX);
    png.extend_from_slice(&len.to_be_bytes());
    png.extend_from_slice(&chunk_type);
    png.extend_from_slice(data);
    let crc = crc32(&chunk_type, data);
    png.extend_from_slice(&crc.to_be_bytes());
}

fn crc32(chunk_type: &[u8], data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &byte in chunk_type.iter().chain(data.iter()) {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB8_8320;
            } else {
                crc >>= 1;
            }
        }
    }
    crc ^ 0xFFFF_FFFF
}

#[allow(clippy::cast_possible_truncation)]
fn deflate_raw(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    // zlib header: CMF=0x78 (deflate, window=32768), FLG=0x01
    out.extend_from_slice(&[0x78, 0x01]);
    let mut offset = 0;
    while offset < data.len() {
        let remaining = data.len() - offset;
        let block_len = remaining.min(65535);
        let is_last = offset + block_len >= data.len();
        out.push(u8::from(is_last));
        out.extend_from_slice(&(block_len as u16).to_be_bytes());
        out.extend_from_slice(&(!block_len as u16).to_be_bytes());
        out.extend_from_slice(&data[offset..offset + block_len]);
        offset += block_len;
    }
    // Adler-32 checksum
    let (mut a, mut b) = (1u32, 0u32);
    for &byte in data {
        a = (a + u32::from(byte)) % 65521;
        b = (b + a) % 65521;
    }
    let adler = (b << 16) | a;
    out.extend_from_slice(&adler.to_be_bytes());
    out
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use easyofd_core::{OfdPage, TextObject};
    use easyofd_writer::OfdWriter;

    /// Helper: create a minimal OFD file at a temp path and return the path.
    fn make_test_ofd(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join("easyofd_tool_tests");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(20.0, 30.0, "CLI smoke test text"));
        let mut w = OfdWriter::new();
        w.add_page(page);
        w.build_to_file(&path).unwrap();
        path
    }

    #[test]
    fn test_info_smoke() {
        let path = make_test_ofd("info_test.ofd");
        cmd_info(&path.to_string_lossy()).unwrap();
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_pages_smoke() {
        let path = make_test_ofd("pages_test.ofd");
        cmd_pages(&path.to_string_lossy()).unwrap();
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_to_markdown_smoke() {
        let path = make_test_ofd("md_test.ofd");
        cmd_to_markdown(&path.to_string_lossy(), None).unwrap();
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_sign_and_verify_smoke() {
        let input = make_test_ofd("sign_test.ofd");
        let output = std::env::temp_dir().join("easyofd_tool_tests/signed_test.ofd");
        cmd_sign(
            &input.to_string_lossy(),
            &output.to_string_lossy(),
            None,
            None,
        )
        .unwrap();
        // Verify the signed file.
        cmd_verify(&output.to_string_lossy()).unwrap();
        let _ = std::fs::remove_file(&input);
        let _ = std::fs::remove_file(&output);
    }

    #[test]
    fn test_info_nonexistent_file() {
        let result = cmd_info("/nonexistent/path/to/file.ofd");
        assert!(result.is_err());
    }

    #[test]
    fn test_to_pdf_smoke() {
        let input = make_test_ofd("pdf_test.ofd");
        let output = std::env::temp_dir().join("easyofd_tool_tests/pdf_test.pdf");
        cmd_to_pdf(&input.to_string_lossy(), &output.to_string_lossy()).unwrap();
        assert!(output.exists());
        let _ = std::fs::remove_file(&input);
        let _ = std::fs::remove_file(&output);
    }
}
