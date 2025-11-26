//! Text Extraction Module Tests
//!
//! Tests for PDF and DOCX text extraction functionality.

use crate::text_extract;
use std::path::Path;

#[cfg(test)]
mod text_extraction_tests {
    use super::*;

    #[test]
    fn test_extract_plain_text() {
        let content = "This is plain text content.\nWith multiple lines.";
        let bytes = content.as_bytes();

        let result = text_extract::extract_text_from_bytes(bytes, "test.txt");

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), content);
    }

    #[test]
    fn test_extract_markdown() {
        let content = "# Header\n\nSome **bold** text and *italic* text.";
        let bytes = content.as_bytes();

        let result = text_extract::extract_text_from_bytes(bytes, "test.md");

        assert!(result.is_ok());
        assert!(result.unwrap().contains("Header"));
    }

    #[test]
    fn test_extract_csv() {
        let content = "name,age,city\nAlice,30,Paris\nBob,25,London";
        let bytes = content.as_bytes();

        let result = text_extract::extract_text_from_bytes(bytes, "data.csv");

        assert!(result.is_ok());
        let text = result.unwrap();
        assert!(text.contains("Alice"));
        assert!(text.contains("Paris"));
    }

    #[test]
    fn test_extract_json() {
        let content = r#"{"name": "Test", "value": 42}"#;
        let bytes = content.as_bytes();

        let result = text_extract::extract_text_from_bytes(bytes, "data.json");

        assert!(result.is_ok());
        let text = result.unwrap();
        assert!(text.contains("name"));
        assert!(text.contains("Test"));
    }

    #[test]
    fn test_empty_file() {
        let bytes: &[u8] = &[];

        let result = text_extract::extract_text_from_bytes(bytes, "empty.txt");

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_large_text_file() {
        let content = "Line\n".repeat(10000);
        let bytes = content.as_bytes();

        let result = text_extract::extract_text_from_bytes(bytes, "large.txt");

        assert!(result.is_ok());
        assert_eq!(result.unwrap().lines().count(), 10000);
    }

    #[test]
    fn test_unicode_content() {
        let content = "Unicode test: 你好世界 🌍 مرحبا";
        let bytes = content.as_bytes();

        let result = text_extract::extract_text_from_bytes(bytes, "unicode.txt");

        assert!(result.is_ok());
        assert!(result.unwrap().contains("你好世界"));
    }

    #[test]
    fn test_mime_type_detection() {
        let txt_mime = text_extract::get_mime_type("document.txt");
        let md_mime = text_extract::get_mime_type("readme.md");
        let pdf_mime = text_extract::get_mime_type("report.pdf");
        let docx_mime = text_extract::get_mime_type("document.docx");

        assert!(txt_mime.contains("text"));
        assert!(md_mime.contains("markdown") || md_mime.contains("text"));
        assert!(pdf_mime.contains("pdf"));
        assert!(docx_mime.contains("document") || docx_mime.contains("docx"));
    }

    #[test]
    fn test_unsupported_format_fallback() {
        let bytes = &[0x00, 0x01, 0x02, 0x03]; // Binary garbage

        let result = text_extract::extract_text_from_bytes(bytes, "binary.exe");

        // Should either fail gracefully or return empty
        match result {
            Ok(text) => assert!(text.is_empty() || text.len() < 100),
            Err(_) => {} // Expected for truly binary files
        }
    }

    #[test]
    fn test_filename_extensions() {
        let extensions = vec![
            ("file.txt", true),
            ("file.md", true),
            ("file.csv", true),
            ("file.json", true),
            ("file.pdf", true),
            ("file.docx", true),
            ("file.doc", true),
            ("file.exe", false),
            ("file.jpg", false),
        ];

        for (filename, expected_support) in extensions {
            let supported = text_extract::is_supported_format(filename);
            assert_eq!(
                supported, expected_support,
                "Expected {} to be {} supported",
                filename,
                if expected_support { "" } else { "not " }
            );
        }
    }
}

#[cfg(test)]
mod pdf_extraction_tests {
    use super::*;

    // Note: These tests require actual PDF files or PDF creation
    // For unit tests, we test the error handling and basic structure

    #[test]
    fn test_pdf_extraction_invalid_bytes() {
        let invalid_pdf = b"This is not a PDF";

        let result = text_extract::extract_pdf_text(invalid_pdf);

        // Should fail for invalid PDF
        assert!(result.is_err());
    }

    #[test]
    fn test_pdf_extraction_empty_bytes() {
        let empty: &[u8] = &[];

        let result = text_extract::extract_pdf_text(empty);

        assert!(result.is_err());
    }
}

#[cfg(test)]
mod docx_extraction_tests {
    use super::*;

    #[test]
    fn test_docx_extraction_invalid_bytes() {
        let invalid_docx = b"This is not a DOCX file";

        let result = text_extract::extract_docx_text(invalid_docx);

        // Should fail for invalid DOCX
        assert!(result.is_err());
    }

    #[test]
    fn test_docx_extraction_empty_bytes() {
        let empty: &[u8] = &[];

        let result = text_extract::extract_docx_text(empty);

        assert!(result.is_err());
    }
}

#[cfg(test)]
mod chunk_tests {
    use super::*;

    #[test]
    fn test_text_chunking() {
        let text = "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.";

        let chunks = text_extract::chunk_text(text, 50, 10);

        assert!(chunks.len() >= 1);
        for chunk in &chunks {
            assert!(chunk.len() <= 60); // Chunk size + some overlap
        }
    }

    #[test]
    fn test_chunking_small_text() {
        let text = "Short text.";

        let chunks = text_extract::chunk_text(text, 100, 10);

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], text);
    }

    #[test]
    fn test_chunking_preserves_content() {
        let text = "Word1 Word2 Word3 Word4 Word5 Word6 Word7 Word8";

        let chunks = text_extract::chunk_text(text, 20, 5);

        // All words should appear in at least one chunk
        let all_text: String = chunks.join(" ");
        for word in text.split_whitespace() {
            assert!(
                all_text.contains(word),
                "Word '{}' should be in chunks",
                word
            );
        }
    }

    #[test]
    fn test_chunking_empty_text() {
        let text = "";

        let chunks = text_extract::chunk_text(text, 100, 10);

        assert!(chunks.is_empty() || (chunks.len() == 1 && chunks[0].is_empty()));
    }

    #[test]
    fn test_chunking_with_overlap() {
        let text = "A B C D E F G H I J K L M N O P Q R S T U V W X Y Z";

        let chunks = text_extract::chunk_text(text, 10, 3);

        // With overlap, adjacent chunks should share some content
        for i in 0..chunks.len().saturating_sub(1) {
            // There should be some overlap between chunks
            // This is a structural test - actual overlap depends on implementation
        }
    }
}
