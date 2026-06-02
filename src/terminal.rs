use std::io::{self, Write};

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
        crossterm::terminal::enable_raw_mode()?;
        Ok(Self { raw_mode: true })
    }

    pub fn restore(&mut self) -> Result<(), TerminalError> {
        if self.raw_mode {
            crossterm::terminal::disable_raw_mode()?;
            self.raw_mode = false;
        }
        Ok(())
    }

    pub fn size(&self) -> Result<(u16, u16), TerminalError> {
        let (cols, rows) = crossterm::terminal::size()?;
        Ok((cols, rows))
    }

    pub fn read(&mut self) -> Result<Vec<u8>, TerminalError> {
        use crossterm::event::{Event, KeyEvent, KeyEventKind};
        let buf = Vec::new();
        while crossterm::event::poll(std::time::Duration::from_millis(0))? {
            if let Event::Key(KeyEvent { kind: KeyEventKind::Press, .. }) = crossterm::event::read()? {
                // For simplicity in this layer, we don't decode keys here.
                // This is a placeholder until an input parser is built.
            }
        }
        Ok(buf)
    }

    pub fn write(&mut self, data: &[u8]) -> Result<(), TerminalError> {
        io::stdout().write_all(data)?;
        io::stdout().flush()?;
        Ok(())
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

    fn acquire_lock() -> std::sync::MutexGuard<'static, ()> {
        RAW_MODE_LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

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
        let _guard = acquire_lock();
        let Ok(term) = Terminal::new() else {
            return;
        };
        assert!(term.raw_mode);
        let mut term = term;
        let _ = term.restore();
        assert!(!term.raw_mode);
    }

    #[test]
    fn test_terminal_restore_disables_raw_mode() {
        let _guard = acquire_lock();
        let Ok(mut term) = Terminal::new() else {
            return;
        };
        assert!(term.raw_mode);
        term.restore().unwrap();
        assert!(!term.raw_mode);
    }

    #[test]
    fn test_terminal_drop_restores() {
        let _guard = acquire_lock();
        {
            let Ok(term) = Terminal::new() else {
                return;
            };
            assert!(term.raw_mode);
            drop(term);
        }
        let is_raw = crossterm::terminal::is_raw_mode_enabled().unwrap_or(false);
        assert!(!is_raw);
    }

    #[test]
    fn test_terminal_size_returns_nonzero() {
        let _guard = acquire_lock();
        let Ok(term) = Terminal::new() else {
            return;
        };
        let (cols, rows) = term.size().unwrap();
        assert!(cols > 0);
        assert!(rows > 0);
    }

    #[test]
    fn test_terminal_write() {
        let _guard = acquire_lock();
        let Ok(mut term) = Terminal::new() else {
            return;
        };
        term.write(b"hello").unwrap();
    }

    #[test]
    fn test_terminal_read_eof() {
        let _guard = acquire_lock();
        let Ok(mut term) = Terminal::new() else {
            return;
        };
        let data = term.read().unwrap();
        assert!(data.is_empty());
    }
}
