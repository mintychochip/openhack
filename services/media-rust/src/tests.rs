#[cfg(test)]
mod tests {
    use crate::errors::AppError;
    use crate::routes::uploads::{
        detect_mime_type, validate_content_type, validate_file_size, ALLOWED_TYPES,
    };

    /// Test that file sizes are validated against MAX_FILE_SIZE.
    ///
    /// # Expected Behavior
    ///
    /// Calls `validate_file_size` with sizes at, below, and above various
    /// `max_size` thresholds. Files at or below the limit return `Ok(())`;
    /// files above the limit return `Err(AppError::FileTooLarge)`. The
    /// boundary value (size == max_size) is accepted (inclusive). A
    /// zero-byte file is always accepted. The default application limit
    /// of 104_857_600 bytes (100 MB) is tested as a specific case.
    ///
    /// # Errors
    ///
    /// Panics if any assertion fails.
    ///
    /// # Side Effects
    ///
    /// None. Pure test assertions with no I/O.
    #[test]
    #[allow(clippy::missing_panics_doc)]
    fn test_file_size_validation() {
        assert!(
            validate_file_size(0, 100).is_ok(),
            "zero-byte file should always be accepted"
        );
        assert!(
            validate_file_size(1, 100).is_ok(),
            "file of size 1 with max 100 should be accepted"
        );
        assert!(
            validate_file_size(99, 100).is_ok(),
            "file of size 99 with max 100 should be accepted"
        );
        assert!(
            validate_file_size(100, 100).is_ok(),
            "file at exact limit should be accepted (inclusive)"
        );
        assert!(
            validate_file_size(101, 100).is_err(),
            "file exceeding limit should be rejected"
        );
        assert!(
            validate_file_size(1000, 100).is_err(),
            "file well over limit should be rejected"
        );

        assert!(
            validate_file_size(104_857_600, 104_857_600).is_ok(),
            "file at default MAX_FILE_SIZE should be accepted"
        );
        assert!(
            validate_file_size(104_857_601, 104_857_600).is_err(),
            "file one byte over default MAX_FILE_SIZE should be rejected"
        );

        let err = validate_file_size(200, 100).unwrap_err();
        assert!(
            matches!(err, AppError::FileTooLarge(_)),
            "over-limit file should produce FileTooLarge error"
        );
    }

    /// Test that MIME types are correctly identified from file extensions.
    ///
    /// # Expected Behavior
    ///
    /// Calls `detect_mime_type` with filenames having common extensions and
    /// verifies the returned MIME type matches the expected value. Tests
    /// image types (jpg, jpeg, png, gif, webp, svg), document types (pdf,
    /// doc, docx, txt), video types (mp4, webm), audio types (mp3, wav),
    /// and fallback to "application/octet-stream" for unknown extensions or
    /// files without extensions. Also verifies case-insensitive matching
    /// (e.g. "PHOTO.JPG" returns "image/jpeg"). Additionally tests
    /// `validate_content_type` against the `ALLOWED_TYPES` category map:
    /// valid content types for a category are accepted, mismatched types
    /// are rejected with `AppError::InvalidFileType`, and unknown categories
    /// accept any content type.
    ///
    /// # Errors
    ///
    /// Panics if any assertion fails.
    ///
    /// # Side Effects
    ///
    /// None. Pure test assertions with no I/O.
    #[test]
    #[allow(clippy::missing_panics_doc)]
    fn test_mime_type_detection() {
        assert_eq!(detect_mime_type("photo.jpg"), "image/jpeg");
        assert_eq!(detect_mime_type("photo.jpeg"), "image/jpeg");
        assert_eq!(detect_mime_type("image.png"), "image/png");
        assert_eq!(detect_mime_type("anim.gif"), "image/gif");
        assert_eq!(detect_mime_type("img.webp"), "image/webp");
        assert_eq!(detect_mime_type("icon.svg"), "image/svg+xml");

        assert_eq!(detect_mime_type("doc.pdf"), "application/pdf");
        assert_eq!(detect_mime_type("file.doc"), "application/msword");
        assert_eq!(
            detect_mime_type("file.docx"),
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        );
        assert_eq!(detect_mime_type("notes.txt"), "text/plain");

        assert_eq!(detect_mime_type("clip.mp4"), "video/mp4");
        assert_eq!(detect_mime_type("clip.webm"), "video/webm");

        assert_eq!(detect_mime_type("song.mp3"), "audio/mpeg");
        assert_eq!(detect_mime_type("sound.wav"), "audio/wav");

        assert_eq!(
            detect_mime_type("unknown.xyz"),
            "application/octet-stream",
            "unknown extension should fall back to octet-stream"
        );
        assert_eq!(
            detect_mime_type("noextension"),
            "application/octet-stream",
            "file without extension should fall back to octet-stream"
        );
        assert_eq!(
            detect_mime_type(""),
            "application/octet-stream",
            "empty filename should fall back to octet-stream"
        );
        assert_eq!(
            detect_mime_type("PHOTO.JPG"),
            "image/jpeg",
            "uppercase extension should be case-insensitive"
        );
        assert_eq!(
            detect_mime_type("Photo.JpG"),
            "image/jpeg",
            "mixed-case extension should be case-insensitive"
        );

        assert!(
            validate_content_type("image/jpeg", "image").is_ok(),
            "image/jpeg should be valid for category 'image'"
        );
        assert!(
            validate_content_type("image/png", "image").is_ok(),
            "image/png should be valid for category 'image'"
        );
        assert!(
            validate_content_type("video/mp4", "video").is_ok(),
            "video/mp4 should be valid for category 'video'"
        );
        assert!(
            validate_content_type("audio/mpeg", "audio").is_ok(),
            "audio/mpeg should be valid for category 'audio'"
        );
        assert!(
            validate_content_type("application/pdf", "document").is_ok(),
            "application/pdf should be valid for category 'document'"
        );
        assert!(
            validate_content_type("text/plain", "document").is_ok(),
            "text/plain should be valid for category 'document'"
        );
        assert!(
            validate_content_type("application/msword", "document").is_ok(),
            "application/msword should be valid for category 'document'"
        );

        assert!(
            validate_content_type("video/mp4", "image").is_err(),
            "video/mp4 should be invalid for category 'image'"
        );
        assert!(
            validate_content_type("image/png", "document").is_err(),
            "image/png should be invalid for category 'document'"
        );
        assert!(
            validate_content_type("audio/wav", "image").is_err(),
            "audio/wav should be invalid for category 'image'"
        );

        assert!(
            validate_content_type("image/jpeg", "nonexistent").is_ok(),
            "unknown category should accept any content type"
        );
        assert!(
            validate_content_type("video/mp4", "nonexistent").is_ok(),
            "unknown category should accept any content type"
        );

        let err = validate_content_type("video/mp4", "image").unwrap_err();
        assert!(
            matches!(err, AppError::InvalidFileType(_)),
            "mismatched content type should produce InvalidFileType error"
        );

        let known_categories: Vec<&&str> = ALLOWED_TYPES.iter().map(|(cat, _)| cat).collect();
        assert!(
            known_categories.contains(&&"image"),
            "ALLOWED_TYPES should contain 'image' category"
        );
        assert!(
            known_categories.contains(&&"document"),
            "ALLOWED_TYPES should contain 'document' category"
        );
        assert!(
            known_categories.contains(&&"video"),
            "ALLOWED_TYPES should contain 'video' category"
        );
        assert!(
            known_categories.contains(&&"audio"),
            "ALLOWED_TYPES should contain 'audio' category"
        );
    }
}
