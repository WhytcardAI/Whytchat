//! Text extraction module for various file formats
//! Supports: TXT, MD, CSV, JSON, PDF, DOCX

use tracing::{info, warn};

/// Extract text content from binary file data based on file extension
pub fn extract_text_from_file(file_name: &str, file_data: &[u8]) -> Result<String, String> {
    let extension = std::path::Path::new(file_name)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    info!("Extracting text from file: {} (type: {})", file_name, extension);

    match extension.as_str() {
        // Plain text formats - direct UTF-8 conversion
        "txt" | "md" | "csv" | "json" => {
            String::from_utf8(file_data.to_vec())
                .map_err(|e| format!("Invalid UTF-8 content: {}", e))
        }

        // PDF extraction
        "pdf" => extract_pdf_text(file_data),

        // DOCX extraction
        "docx" | "doc" => extract_docx_text(file_data),

        _ => Err(format!("Unsupported file extension: {}", extension)),
    }
}

/// Extract text from PDF file
fn extract_pdf_text(file_data: &[u8]) -> Result<String, String> {
    info!("Extracting text from PDF...");

    match pdf_extract::extract_text_from_mem(file_data) {
        Ok(text) => {
            let cleaned = clean_extracted_text(&text);
            info!("PDF extraction successful: {} characters", cleaned.len());
            Ok(cleaned)
        }
        Err(e) => {
            warn!("PDF extraction failed: {}", e);
            Err(format!("Failed to extract PDF text: {}", e))
        }
    }
}

/// Extract text from DOCX file
fn extract_docx_text(file_data: &[u8]) -> Result<String, String> {
    info!("Extracting text from DOCX...");

    match docx_rs::read_docx(file_data) {
        Ok(docx) => {
            let mut text_parts: Vec<String> = Vec::new();

            // Extract text from document body
            for child in docx.document.children {
                if let docx_rs::DocumentChild::Paragraph(para) = child {
                    let para_text: String = para.children.iter()
                        .filter_map(|pc| {
                            if let docx_rs::ParagraphChild::Run(run) = pc {
                                Some(run.children.iter()
                                    .filter_map(|rc| {
                                        if let docx_rs::RunChild::Text(t) = rc {
                                            Some(t.text.clone())
                                        } else {
                                            None
                                        }
                                    })
                                    .collect::<Vec<_>>()
                                    .join(""))
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("");

                    if !para_text.trim().is_empty() {
                        text_parts.push(para_text);
                    }
                }
            }

            let text = text_parts.join("\n");
            let cleaned = clean_extracted_text(&text);
            info!("DOCX extraction successful: {} characters", cleaned.len());
            Ok(cleaned)
        }
        Err(e) => {
            warn!("DOCX extraction failed: {}", e);
            Err(format!("Failed to extract DOCX text: {}", e))
        }
    }
}

/// Clean up extracted text
fn clean_extracted_text(text: &str) -> String {
    text.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_txt_extraction() {
        let content = b"Hello, World!\nThis is a test.";
        let result = extract_text_from_file("test.txt", content);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, World!\nThis is a test.");
    }

    #[test]
    fn test_md_extraction() {
        let content = b"# Title\n\nThis is **markdown** content.\n\n- Item 1\n- Item 2";
        let result = extract_text_from_file("readme.md", content);
        assert!(result.is_ok());
        let text = result.unwrap();
        assert!(text.contains("# Title"));
        assert!(text.contains("**markdown**"));
    }

    #[test]
    fn test_csv_extraction() {
        let content = b"name,age,city\nAlice,30,Paris\nBob,25,London";
        let result = extract_text_from_file("data.csv", content);
        assert!(result.is_ok());
        let text = result.unwrap();
        assert!(text.contains("Alice"));
        assert!(text.contains("Paris"));
    }

    #[test]
    fn test_json_extraction() {
        let content = b"{\"name\": \"Test\", \"value\": 42}";
        let result = extract_text_from_file("config.json", content);
        assert!(result.is_ok());
        let text = result.unwrap();
        assert!(text.contains("Test"));
        assert!(text.contains("42"));
    }

    #[test]
    fn test_unsupported_extension() {
        let content = b"Some binary data";
        let result = extract_text_from_file("test.xyz", content);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unsupported"));
    }

    #[test]
    fn test_empty_file() {
        let content = b"";
        let result = extract_text_from_file("empty.txt", content);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }

    #[test]
    fn test_french_content() {
        let content = "Bonjour le monde!\nCeci est un test avec des caractères spéciaux: é, è, à, ç".as_bytes();
        let result = extract_text_from_file("french.txt", content);
        assert!(result.is_ok());
        let text = result.unwrap();
        assert!(text.contains("Bonjour"));
        assert!(text.contains("é"));
        assert!(text.contains("ç"));
    }

    #[test]
    fn test_clean_extracted_text() {
        let dirty = "  Line 1  \n\n  Line 2  \n   \n  Line 3  ";
        let cleaned = clean_extracted_text(dirty);
        assert_eq!(cleaned, "Line 1\nLine 2\nLine 3");
    }
}
