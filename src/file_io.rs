use std::path::Path;

#[derive(Debug, PartialEq)]
pub enum FileError {
    NotFound,
    PermissionDenied,
    InvalidUtf8,
    WriteFailed,
}

pub fn read_file(path: &Path) -> Result<String, FileError> {
    let bytes = std::fs::read(path).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => FileError::NotFound,
        std::io::ErrorKind::PermissionDenied => FileError::PermissionDenied,
        _ => FileError::PermissionDenied,
    })?;
    let mut text = String::from_utf8(bytes).map_err(|_| FileError::InvalidUtf8)?;
    if text.contains('\r') {
        let mut normalized = String::with_capacity(text.len());
        let mut chars = text.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '\r' {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                normalized.push('\n');
            } else {
                normalized.push(ch);
            }
        }
        text = normalized;
    }
    Ok(text)
}

pub fn write_file(path: &Path, contents: &str) -> Result<(), FileError> {
    if path.exists() {
        let mut backup = path.as_os_str().to_os_string();
        backup.push("~");
        std::fs::rename(path, &backup).map_err(|_| FileError::WriteFailed)?;
    }
    std::fs::write(path, contents).map_err(|_| FileError::WriteFailed)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;
    use std::fs;

    #[test]
    fn test_read_file_not_found() {
        let path = temp_dir().join("dim_test_nonexistent_file.txt");
        let result = read_file(&path);
        assert_eq!(result, Err(FileError::NotFound));
    }

    #[test]
    fn test_read_file_empty() {
        let path = temp_dir().join("dim_test_empty.txt");
        fs::write(&path, "").unwrap();
        let result = read_file(&path).unwrap();
        assert_eq!(result, "");
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_read_file_utf8_japanese() {
        let path = temp_dir().join("dim_test_japanese.txt");
        fs::write(&path, "こんにちは世界").unwrap();
        let result = read_file(&path).unwrap();
        assert_eq!(result, "こんにちは世界");
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_read_file_normalizes_line_endings() {
        let path = temp_dir().join("dim_test_lines.txt");
        fs::write(&path, "line1\r\nline2\nline3\r\n").unwrap();
        let result = read_file(&path).unwrap();
        assert_eq!(result, "line1\nline2\nline3\n");
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_write_file_creates_file() {
        let path = temp_dir().join("dim_test_write.txt");
        let result = write_file(&path, "hello");
        assert!(result.is_ok());
        let contents = fs::read_to_string(&path).unwrap();
        assert_eq!(contents, "hello");
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_write_file_creates_backup() {
        let path = temp_dir().join("dim_test_backup.txt");
        fs::write(&path, "original").unwrap();
        let backup_path = temp_dir().join("dim_test_backup.txt~");
        let result = write_file(&path, "updated");
        assert!(result.is_ok());
        let backup_contents = fs::read_to_string(&backup_path).unwrap();
        assert_eq!(backup_contents, "original");
        let contents = fs::read_to_string(&path).unwrap();
        assert_eq!(contents, "updated");
        fs::remove_file(&path).unwrap();
        fs::remove_file(&backup_path).unwrap();
    }

    #[test]
    fn test_write_and_read_roundtrip() {
        let path = temp_dir().join("dim_test_roundtrip.txt");
        let text = "日本語テキスト\n改行あり\r\n混在";
        write_file(&path, text).unwrap();
        let result = read_file(&path).unwrap();
        assert_eq!(result, "日本語テキスト\n改行あり\n混在");
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_read_file_invalid_utf8() {
        let path = temp_dir().join("dim_test_invalid_utf8.txt");
        fs::write(&path, vec![0x80, 0x81, 0x82]).unwrap();
        let result = read_file(&path);
        assert_eq!(result, Err(FileError::InvalidUtf8));
        fs::remove_file(&path).unwrap();
    }
}
