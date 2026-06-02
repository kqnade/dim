#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    // Movement
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    PageUp,
    PageDown,
    MoveWordForward,
    MoveWordBackward,
    MoveLineStart,
    MoveLineEnd,
    MoveFileStart,
    MoveFileEnd,

    // Selection
    ExtendSelectionLeft,
    ExtendSelectionRight,
    ExtendSelectionUp,
    ExtendSelectionDown,
    CollapseSelection,
    VisualMode,
    SelectLine,

    // Editing
    DeleteSelection,
    ChangeSelection,
    YankSelection,
    PasteBefore,
    PasteAfter,
    OpenLineAbove,
    OpenLineBelow,

    // Mode transitions
    EnterNormalMode,
    EnterInsertMode,
    EnterCommandMode,
    EnterSearchMode,

    // File / buffer
    OpenFile(String),
    SaveFile,
    SaveFileAs(String),
    Quit,
    ForceQuit,

    // Search
    SearchForward(String),
    SearchBackward(String),
    SearchNext,
    SearchPrevious,

    // Undo / redo
    Undo,
    Redo,

    // SKK
    SkkToggle,
    SkkConfirm,
    SkkCancel,
    SkkNextCandidate,
    SkkPreviousCandidate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseCommandError {
    UnknownCommand(String),
    MissingArgument(String),
}

pub struct CommandRegistry {
    commands: Vec<(&'static str, Command)>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: vec![
                ("write", Command::SaveFile),
                ("w", Command::SaveFile),
                ("quit", Command::Quit),
                ("q", Command::Quit),
                ("wq", Command::SaveFile), // Simplified: wq just saves for now
                ("force-quit", Command::ForceQuit),
                ("q!", Command::ForceQuit),
                ("open", Command::OpenFile(String::new())),
                ("undo", Command::Undo),
                ("redo", Command::Redo),
            ],
        }
    }

    pub fn parse(&self, input: &str) -> Result<Command, ParseCommandError> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(ParseCommandError::UnknownCommand("".to_string()));
        }

        let mut parts = trimmed.splitn(2, ' ');
        let name = parts.next().unwrap();
        let arg = parts.next();

        for (cmd_name, template) in &self.commands {
            if *cmd_name == name {
                return match template {
                    Command::OpenFile(_) => {
                        let path = arg.ok_or_else(|| {
                            ParseCommandError::MissingArgument("open requires a file path".to_string())
                        })?;
                        Ok(Command::OpenFile(path.to_string()))
                    }
                    Command::SaveFileAs(_) => {
                        let path = arg.ok_or_else(|| {
                            ParseCommandError::MissingArgument("save-as requires a file path".to_string())
                        })?;
                        Ok(Command::SaveFileAs(path.to_string()))
                    }
                    _ => Ok(template.clone()),
                };
            }
        }

        Err(ParseCommandError::UnknownCommand(name.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_variants_exist() {
        let cmds = vec![
            Command::MoveLeft,
            Command::MoveRight,
            Command::MoveUp,
            Command::MoveDown,
            Command::PageUp,
            Command::PageDown,
            Command::MoveWordForward,
            Command::MoveWordBackward,
            Command::MoveLineStart,
            Command::MoveLineEnd,
            Command::MoveFileStart,
            Command::MoveFileEnd,
            Command::EnterInsertMode,
            Command::EnterNormalMode,
            Command::EnterCommandMode,
            Command::EnterSearchMode,
            Command::VisualMode,
            Command::SelectLine,
            Command::DeleteSelection,
            Command::ChangeSelection,
            Command::YankSelection,
            Command::PasteBefore,
            Command::PasteAfter,
            Command::OpenLineAbove,
            Command::OpenLineBelow,
            Command::Undo,
            Command::Redo,
            Command::Quit,
            Command::ForceQuit,
            Command::OpenFile("test.txt".to_string()),
            Command::SaveFile,
            Command::SearchForward("hello".to_string()),
            Command::SkkToggle,
        ];
        assert!(!cmds.is_empty());
    }

    #[test]
    fn test_registry_parse_write() {
        let reg = CommandRegistry::new();
        assert_eq!(reg.parse("write"), Ok(Command::SaveFile));
        assert_eq!(reg.parse("w"), Ok(Command::SaveFile));
    }

    #[test]
    fn test_registry_parse_quit() {
        let reg = CommandRegistry::new();
        assert_eq!(reg.parse("quit"), Ok(Command::Quit));
        assert_eq!(reg.parse("q"), Ok(Command::Quit));
    }

    #[test]
    fn test_registry_parse_force_quit() {
        let reg = CommandRegistry::new();
        assert_eq!(reg.parse("q!"), Ok(Command::ForceQuit));
    }

    #[test]
    fn test_registry_parse_open_file() {
        let reg = CommandRegistry::new();
        assert_eq!(
            reg.parse("open file.txt"),
            Ok(Command::OpenFile("file.txt".to_string()))
        );
    }

    #[test]
    fn test_registry_parse_open_missing_arg() {
        let reg = CommandRegistry::new();
        assert_eq!(
            reg.parse("open"),
            Err(ParseCommandError::MissingArgument(
                "open requires a file path".to_string()
            ))
        );
    }

    #[test]
    fn test_registry_parse_unknown() {
        let reg = CommandRegistry::new();
        assert_eq!(
            reg.parse("foobar"),
            Err(ParseCommandError::UnknownCommand("foobar".to_string()))
        );
    }

    #[test]
    fn test_registry_parse_empty() {
        let reg = CommandRegistry::new();
        assert_eq!(
            reg.parse(""),
            Err(ParseCommandError::UnknownCommand("".to_string()))
        );
    }

    #[test]
    fn test_registry_parse_undo() {
        let reg = CommandRegistry::new();
        assert_eq!(reg.parse("undo"), Ok(Command::Undo));
    }

    #[test]
    fn test_registry_parse_redo() {
        let reg = CommandRegistry::new();
        assert_eq!(reg.parse("redo"), Ok(Command::Redo));
    }
}
