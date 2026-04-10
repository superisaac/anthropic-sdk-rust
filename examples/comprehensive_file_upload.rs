use anthropic_sdk::{
    default_retry,
    types::{ContentBlockParam, MessageContent},
    File, FileBuilder, FileConstraints, FileSource, TokenCounter,
};
use std::io::Write;
use tempfile::NamedTempFile;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📁 Comprehensive File Upload Demo");
    println!("=================================\n");

    // Initialize infrastructure
    let token_counter = TokenCounter::new();
    let retry_executor = default_retry();

    // Demo file contents (simulated files for demonstration)
    let image_data = create_sample_image_data();
    let csv_data = create_sample_csv_data();
    let text_data = "This is a sample text document with some content for analysis.";
    let json_data = r#"{"users": [{"name": "Alice", "age": 30}, {"name": "Bob", "age": 25}]}"#;

    println!("🎯 Demo Scenarios:");
    println!("==================\n");

    // Scenario 1: Creating files from different sources
    println!("📂 Scenario 1: File Creation from Multiple Sources");
    println!("--------------------------------------------------");

    // From bytes (name, bytes, mime_type)
    let text_file = File::from_bytes("sample.txt", text_data.as_bytes(), None)?;
    println!(
        "✅ Created text file from bytes: {} ({} bytes)",
        text_file.name, text_file.size
    );

    // From base64 (name, base64_data, mime_type)
    use base64::engine::general_purpose;
    use base64::Engine as _;
    let image_base64 = general_purpose::STANDARD.encode(&image_data);
    let image_file = File::from_base64("sample.png", &image_base64, None)?;
    println!(
        "✅ Created image file from base64: {} ({} bytes)",
        image_file.name, image_file.size
    );

    // Using file builder for complex scenarios
    let csv_file = FileBuilder::new()
        .name("data.csv")
        .with_hash()
        .build(FileSource::Bytes(csv_data.as_bytes().to_vec().into()))
        .await?;
    println!(
        "✅ Created CSV file with builder: {} (hash: {})",
        csv_file.name,
        csv_file
            .hash
            .as_deref()
            .map(|h| format!("{:.8}...", h))
            .unwrap_or_else(|| "none".to_string())
    );

    // Using File::from_bytes for JSON
    let json_file = File::from_bytes("data.json", json_data.as_bytes(), None)?;
    println!(
        "✅ Created JSON file: {} ({} bytes)",
        json_file.name, json_file.size
    );

    // Scenario 2: File validation and constraints
    println!("\n🔒 Scenario 2: File Validation and Constraints");
    println!("----------------------------------------------");

    // Create constraints for different file types
    let image_constraints = FileConstraints {
        max_size: 5 * 1024 * 1024, // 5MB
        allowed_types: Some(vec![
            "image/png".parse().unwrap(),
            "image/jpeg".parse().unwrap(),
        ]),
        require_hash: false,
    };

    let text_constraints = FileConstraints {
        max_size: 1024 * 1024, // 1MB
        allowed_types: Some(vec![
            "text/plain".parse().unwrap(),
            "text/csv".parse().unwrap(),
        ]),
        require_hash: false,
    };

    // Validate files against constraints
    match image_file.validate(&image_constraints) {
        Ok(_) => println!("✅ Image file passed validation"),
        Err(e) => println!("❌ Image file failed validation: {}", e),
    }

    match text_file.validate(&text_constraints) {
        Ok(_) => println!("✅ Text file passed validation"),
        Err(e) => println!("❌ Text file failed validation: {}", e),
    }

    // Test constraint violations
    let large_file_data = vec![0u8; 2 * 1024 * 1024]; // 2MB
    let large_file = File::from_bytes("large.txt", large_file_data, None)?;

    match large_file.validate(&text_constraints) {
        Ok(_) => println!("❌ Large file unexpectedly passed validation"),
        Err(e) => println!("✅ Large file correctly failed validation: {}", e),
    }

    // Scenario 3: File type detection
    println!("\n🕵️ Scenario 3: File Type Detection");
    println!("----------------------------------");

    // Test different file extensions
    let test_files = vec![
        ("document.pdf", "application/pdf"),
        ("image.jpg", "image/jpeg"),
        ("data.txt", "text/plain"),
        ("data.json", "application/json"),
        ("archive.zip", "application/zip"),
    ];

    for (filename, expected_mime) in test_files {
        let test_file = File::from_bytes(filename, b"test data".as_ref(), None)?;
        let matches = test_file.mime_type.to_string() == expected_mime;
        println!(
            "  {} -> {} {}",
            filename,
            test_file.mime_type,
            if matches { "✅" } else { "❌" }
        );
    }

    // Scenario 4: File processing utilities
    println!("\n⚙️ Scenario 4: File Processing Utilities");
    println!("----------------------------------------");

    // Test file type checking
    println!("File type checks:");
    println!("  {} is image: {}", image_file.name, image_file.is_image());
    println!("  {} is text: {}", text_file.name, text_file.is_text());
    println!(
        "  {} is application: {}",
        json_file.name,
        json_file.is_application()
    );

    // Format conversion
    println!("\nFormat conversion:");
    let text_as_base64 = text_file.to_base64().await?;
    println!("  Text file as base64: {:.50}...", text_as_base64);

    let image_as_bytes = image_file.to_bytes().await?;
    println!("  Image file as bytes: {} bytes", image_as_bytes.len());

    // Scenario 5: Temporary files and cleanup
    println!("\n🗑️ Scenario 5: Temporary Files and Cleanup");
    println!("-------------------------------------------");

    // Create a temporary file
    let mut temp_file = NamedTempFile::new()?;
    temp_file.write_all(b"Temporary file content for testing")?;
    temp_file.flush()?;

    let temp_path = temp_file.path().to_path_buf();
    println!("Created temporary file: {}", temp_path.display());

    // Create File from path
    let file_from_path = File::from_path(&temp_path)?;
    println!(
        "✅ Loaded file from path: {} ({} bytes)",
        file_from_path.name, file_from_path.size
    );

    // File will be automatically cleaned up when temp_file is dropped
    drop(temp_file);
    println!("✅ Temporary file cleaned up automatically");

    // Scenario 6: Message integration examples
    println!("\n💬 Scenario 6: Message Integration");
    println!("----------------------------------");

    // Create content blocks with file attachments
    let image_content = ContentBlockParam::image_file(image_file.clone()).await?;
    println!("✅ Created image content block for message");

    // Create content from file using convenience method
    let file_content = ContentBlockParam::from_file(json_file.clone()).await?;
    println!("✅ Created file content block from JSON file");

    // Build a multi-part message with files
    let message_content = MessageContent::Blocks(vec![
        ContentBlockParam::text("Please analyze these files:"),
        ContentBlockParam::text("1. Sample image:"),
        image_content,
        ContentBlockParam::text("2. Data file:"),
        file_content,
        ContentBlockParam::text("What insights can you provide?"),
    ]);

    println!(
        "✅ Built multi-part message with {} content blocks",
        if let MessageContent::Blocks(ref blocks) = message_content {
            blocks.len()
        } else {
            0
        }
    );

    // Scenario 7: Performance and memory efficiency
    println!("\n⚡ Scenario 7: Performance Metrics");
    println!("----------------------------------");

    let start_time = std::time::Instant::now();

    // Process multiple files in parallel (simulated)
    let files = vec![&text_file, &image_file, &csv_file, &json_file];
    let total_size: u64 = files.iter().map(|f| f.size).sum();
    let total_files = files.len();

    let processing_time = start_time.elapsed();

    println!("Performance metrics:");
    println!("  Files processed: {}", total_files);
    println!("  Total data size: {} bytes", total_size);
    println!("  Processing time: {:?}", processing_time);
    if processing_time.as_secs_f64() > 0.0 {
        println!(
            "  Throughput: {:.2} MB/s",
            (total_size as f64 / 1024.0 / 1024.0) / processing_time.as_secs_f64()
        );
    }

    // Usage tracking
    let usage_summary = token_counter.get_summary();
    println!("\nInfrastructure metrics:");
    println!(
        "  Session duration: {:.1} seconds",
        usage_summary.session_duration.as_secs_f64()
    );
    println!(
        "  Retry policy: {} max retries",
        retry_executor.get_policy().max_retries
    );

    println!("\n📊 File Summary");
    println!("===============");

    let all_files = vec![
        ("Text", &text_file),
        ("Image", &image_file),
        ("CSV", &csv_file),
        ("JSON", &json_file),
    ];

    for (type_name, file) in all_files {
        println!("{} File:", type_name);
        println!("  Name: {}", file.name);
        println!("  Size: {} bytes", file.size);
        println!("  MIME: {}", file.mime_type);
        println!("  Hash: {}", file.hash.as_deref().unwrap_or("none"));
        println!();
    }

    println!("✨ Comprehensive File Upload Demo Complete!");
    println!("🚀 All file scenarios executed successfully!");
    println!("💡 This demonstrates production-ready file handling with:");
    println!("   • Multiple file sources (bytes, base64, paths, builder)");
    println!("   • Comprehensive validation and constraints");
    println!("   • MIME type detection and processing utilities");
    println!("   • Message integration for AI workflows");
    println!("   • Performance optimization and memory efficiency");

    Ok(())
}

/// Create sample image data (PNG header + minimal data)
fn create_sample_image_data() -> Vec<u8> {
    // PNG signature + minimal PNG structure for demo
    vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, // IHDR chunk length
        0x49, 0x48, 0x44, 0x52, // IHDR
        0x00, 0x00, 0x00, 0x01, // Width: 1
        0x00, 0x00, 0x00, 0x01, // Height: 1
        0x08, 0x06, 0x00, 0x00, 0x00, // Bit depth, color type, etc.
        0x1F, 0x15, 0xC4, 0x89, // CRC
        0x00, 0x00, 0x00, 0x00, // IEND chunk length
        0x49, 0x45, 0x4E, 0x44, // IEND
        0xAE, 0x42, 0x60, 0x82, // CRC
    ]
}

/// Create sample CSV data
fn create_sample_csv_data() -> String {
    "name,age,city\n\
     Alice,30,San Francisco\n\
     Bob,25,New York\n\
     Carol,35,London\n\
     David,28,Tokyo"
        .to_string()
}
