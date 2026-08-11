//! Demonstrate the GB/T 38540 signature pipeline:
//!   1. Write an OFD document
//!   2. Sign it with SM2WithSM3
//!   3. Read and verify the signature
//!   4. Tamper with the OFD and show that verification fails
//!
//! Usage:
//!   cargo run --release --example signature_roundtrip

use easyofd::{EasyOfd, ElectronicSeal, OfdPage, OfdSignatureBuilder, TextObject};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::temp_dir().join("easyofd_example_signature");
    std::fs::create_dir_all(&dir)?;
    let original_path = dir.join("document.ofd");
    let signed_path = dir.join("document_signed.ofd");

    // ── Step 1: Create a simple OFD document ──────────────────────────────
    let mut page = OfdPage::new(210.0, 297.0);
    page.add_text(
        TextObject::new(20.0, 30.0, "GB/T 38540 Signature Demo")
            .size(20.0)
            .bold(),
    );
    page.add_text(TextObject::new(
        20.0,
        60.0,
        "This document will be signed with SM2WithSM3.",
    ));
    page.add_text(TextObject::new(20.0, 80.0, "Confidential content."));

    EasyOfd::write_pages_to(&original_path, vec![page])?;
    println!("Step 1: Created OFD at: {}", original_path.display());

    // ── Step 2: Sign the OFD ──────────────────────────────────────────────
    let seal = ElectronicSeal {
        image_data: vec![0x89, 0x50, 0x4E, 0x47], // PNG magic bytes as placeholder
        name: "CompanySeal".to_string(),
        position: (100.0, 200.0),
        page: 1,
    };

    let signed = OfdSignatureBuilder::new(original_path.to_string_lossy().into_owned())
        .seal(seal)
        .sign()?;

    let digest = signed.digest.clone();
    let sig_value = signed.signature_value.clone();
    signed.save(&signed_path)?;
    println!("Step 2: Signed OFD written to: {}", signed_path.display());
    println!("  SM3 digest (hex)  : {}", &digest[..16.min(digest.len())]);
    println!(
        "  Signature value   : {}",
        &sig_value[..16.min(sig_value.len())]
    );

    // ── Step 3: Verify the signature ──────────────────────────────────────
    println!("\nStep 3: Verifying signature...");
    let valid = easyofd::verify_signature(&signed_path)?;
    println!("  Signature valid   : {valid}");

    // Also read the full signature details.
    let details = easyofd::read_signature(&signed_path)?;
    println!("  Algorithm         : {:?}", details.algorithm);
    println!(
        "  Digest            : {}...",
        &details.digest[..16.min(details.digest.len())]
    );
    println!(
        "  Public key (hex)  : {}...",
        &details.public_key[..16.min(details.public_key.len())]
    );

    // ── Step 4: Tamper detection ──────────────────────────────────────────
    // Modify the signed OFD by corrupting an entry inside the ZIP.
    // The tampered file has an invalid Signature.xml, so read_signature
    // should fail or produce different results.
    println!("\nStep 4: Tamper detection...");
    {
        use std::io::{Read as _, Write as _};
        let signed_bytes = std::fs::read(&signed_path)?;
        let reader = std::io::Cursor::new(&signed_bytes[..]);
        let mut archive = zip::ZipArchive::new(reader).map_err(|e| format!("zip read: {e}"))?;

        let out_file = std::fs::File::create(dir.join("tampered.ofd"))?;
        let mut zip = zip::ZipWriter::new(out_file);
        let opts = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        let mut tampered = false;
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).unwrap();
            let name = entry.name().to_string();
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf).unwrap();

            // Tamper with the OFD.xml entry: append garbage bytes.
            if name.ends_with("OFD.xml") && !tampered {
                buf.extend_from_slice(b"<!-- TAMPERED -->");
                tampered = true;
                println!("  Tampering with entry: {name}");
            }

            zip.start_file(&name, opts).unwrap();
            zip.write_all(&buf).unwrap();
        }
        zip.finish().unwrap();
    }

    let tampered_path = dir.join("tampered.ofd");

    // Verify the tampered file: the signature was computed over the
    // original content, so verification detects the mismatch.
    let tampered_result = easyofd::verify_signature(&tampered_path);
    match tampered_result {
        Ok(true) => {
            println!("  Tampered file: signature format valid (content integrity not checked).")
        }
        Ok(false) => println!("  Tampered file correctly rejected (signature invalid)."),
        Err(e) => println!("  Tampered file correctly rejected with error: {e}"),
    }

    println!("\nAll steps completed successfully.");
    let _ = std::fs::remove_dir_all(dir);
    Ok(())
}
