use std::io::{self, Read, Write};

#[derive(Debug)]
pub enum TerminalError {
    IoError(io::Error),
    UnsupportedTerminal,
}

impl std::fmt::Display for TerminalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TerminalError::IoError(e) => write!(f, "IO error: {e}"),
            TerminalError::UnsupportedTerminal => write!(f, "Unsupported terminal"),
        }
    }
}

impl std::error::Error for TerminalError {}

impl From<io::Error> for TerminalError {
    fn from(e: io::Error) -> Self {
        TerminalError::IoError(e)
    }
}

pub struct Terminal {
    raw_mode: bool,
}

impl Terminal {
    pub fn new() -> Result<Self, TerminalError> {
        todo!()
    }

    pub fn restore(&mut self) -> Result<(), TerminalError> {
        todo!()
    }

    pub fn size(&self) -> Result<(u16, u16), TerminalError> {
        todo!()
    }

    pub fn read(&mut self) -> Result<Vec<u8>, TerminalError> {
        todo!()
    }

    pub fn write(&mut self, _data: &[u8]) -> Result<(), TerminalError> {
        todo!()
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static RAW_MODE_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_terminal_error_display_io() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "test");
        let err = TerminalError::from(io_err);
        assert!(err.to_string().contains("IO error"));
    }

    #[test]
    fn test_terminal_error_display_unsupported() {
        let err = TerminalError::UnsupportedTerminal;
        assert_eq!(err.to_string(), "Unsupported terminal");
    }

    #[test]
    fn test_terminal_new_sets_raw_mode() {
        let _guard = RAW_MODE_LOCK.lock().unwrap();
        let term = Terminal::new().unwrap();
        assert!(term.raw_mode);
        let mut term = term;
        term.restore().unwrap();
        assert!(!term.raw_mode);
    }

    #[test]
    fn test_terminal_restore_disables_raw_mode() {
        let _guard = RAW_MODE_LOCK.lock().unwrap();
        let mut term = Terminal::new().unwrap();
        assert!(term.raw_mode);
        term.restore().unwrap();
        assert!(!term.raw_mode);
    }

    #[test]
    fn test_terminal_drop_restores() {
        let _guard = RAW_MODE_LOCK.lock().unwrap();
        {
            let term = Terminal::new().unwrap();
            assert!(term.raw_mode);
            drop(term);
        }
        assert!(!crossterm::terminal::is_raw_mode_enabled().unwrap());
    }

    #[test]
    fn test_terminal_size_returns_nonzero() {
        let _guard = RAW_MODE_LOCK.lock().unwrap();
        let term = Terminal::new().unwrap();
        let (cols, rows) = term.size().unwrap();
        assert!(cols > 0);
        assert!(rows > 0);
    }

    #[test]
    fn test_terminal_write() {
        let _guard = RAW_MODE_LOCK.lock().unwrap();
        let mut term = Terminal::new().unwrap();
        term.write(b"hello").unwrap();
    }

    #[test]
    fn test_terminal_read_eof() {
        let _guard = RAW_MODE_LOCK.lock().unwrap();
        let mut term = Terminal::new().unwrap();
        let data = term.read().unwrap();
        assert!(data.is_empty());
    }
}
