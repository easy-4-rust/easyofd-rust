# ofdrw Expected Products

This directory contains **hand-crafted expected XML products** representing the
output that the [ofdrw](https://github.com/ofdrw/ofdrw) Java implementation
would produce for each test fixture.

## Purpose

These files enable **L2 structural comparison** between easyofd-rust and ofdrw
without requiring a JDK environment. They document the expected XML structure
that a conforming OFD writer (like ofdrw) should produce per GB/T 33190-2016.

## Files

Each fixture has three expected XML files:

| Fixture | OFD.xml | Document.xml | Page Content.xml |
|---------|---------|--------------|------------------|
| simple_1.ofd | `simple_1_ofd.xml` | `simple_1_document.xml` | `simple_1_page_0.xml` |
| simple_2.ofd | `simple_2_ofd.xml` | `simple_2_document.xml` | `simple_2_page_0.xml` |
| multi_page_image.ofd | `multi_page_image_ofd.xml` | `multi_page_image_document.xml` | `multi_page_image_page_0.xml` |
| signed.ofd | `signed_ofd.xml` | `signed_document.xml` | `signed_page_0.xml` |
| with_table.ofd | `with_table_ofd.xml` | `with_table_document.xml` | `with_table_page_0.xml` |

## How These Were Constructed

These expected products were derived from:

1. **ofdrw source code** (fetched from GitHub):
   - `OFD.java`: Root element uses `Version` and `DocType` attributes
   - `CT_DocInfo.java`: DocID (UUID without dashes), CreationDate, Creator, CreatorVersion
   - `Document.java`: CommonData (MaxUnitID, PageArea, PublicRes, DocumentRes) + Pages
   - `Pages.java` / `Page.java`: Page tree with ID and BaseLoc attributes

2. **GB/T 33190-2016 standard**: XML namespace, element hierarchy, required attributes

3. **Actual fixture metadata** (from `baseline/expected_*.json`):
   - Page counts, image/path counts, signature presence
   - PhysicalBox dimensions extracted from real fixtures

## Limitations

**These are NOT byte-identical copies of ofdrw output.** They are structural
approximations based on:

- The ofdrw source code (which reveals the XML template structure)
- The GB/T 33190-2016 specification
- Known ofdrw conventions (e.g., `Doc_0/` prefix, `Page_N/` naming)

A genuine byte-level comparison requires:

1. A JDK 8+ environment
2. Maven to build ofdrw from source
3. Running `mvn -pl ofdrw-full test` to produce actual OFD output
4. Comparing the binary output against these expected files

See `scripts/verify_with_ofdrw.sh` for the CI-ready verification procedure.

## ofdrw Test Samples (from GitHub)

The ofdrw project uses these test OFD samples:

### ofdrw-reader test resources
- `helloworld.ofd` - Basic single-page document with Chinese text
- `helloworld_with_pageblock.ofd` - Same content wrapped in PageBlock
- `chineseDir.ofd` - Document with Chinese directory structure
- `keyword.ofd` - Document for keyword search testing
- `keyword2.ofd` - Additional keyword search test
- `multiKeywordInTextCode.ofd` - Multiple keywords in text codes
- `path_unstd.ofd` - Non-standard path data
- `AddAttachment.ofd` - Document with attachments
- `SESV4SignDoc.ofd` - Signed document (SES V4)
- `发票示例.ofd` - Invoice example

### ofdrw-layout test resources
- `helloworld.ofd` - Generated HelloWorld document
- `1-1.ofd` - Basic layout test
- `Page5.ofd` - Multi-page layout
- `SplitParagraph.ofd` - Paragraph splitting test
- `AddWatermarkAnnot.ofd` - Watermark annotation test
- `areaholder_fields.ofd` - Area holder fields
- `fptpl.ofd` - Form template
- `keyword.ofd` / `keyword2.ofd` - Keyword search tests
- `no_page_container.ofd` - Document without page container
- `拿来主义_page6.ofd` - Chinese-named page test
