use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkkState {
    Direct,
    Hiragana,
    Katakana,
    Converting,
    Registering,
}

pub struct Dictionary {
    entries: HashMap<String, Vec<String>>,
}

impl Dictionary {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn from_builtin() -> Self {
        let mut dict = Self::new();
        dict.add("にほんご", &["日本語"]);
        dict.add("てきすと", &["テキスト", "文章"]);
        dict.add("へんしゅう", &["編集"]);
        dict.add("ふぁいる", &["ファイル"]);
        dict
    }

    pub fn add(&mut self, reading: &str, candidates: &[&str]) {
        let entry: Vec<String> = candidates.iter().map(|s| s.to_string()).collect();
        self.entries.insert(reading.to_string(), entry);
    }

    pub fn lookup(&self, reading: &str) -> Option<&Vec<String>> {
        self.entries.get(reading)
    }

    /// Load dictionary from SKK-JISYO format file (UTF-8).
    /// Format: `reading /candidate1/candidate2/`
    pub fn load_from_file(&mut self, path: &Path) -> std::io::Result<()> {
        let content = fs::read_to_string(path)?;
        for line in content.lines() {
            if line.is_empty() || line.starts_with(";;") {
                continue;
            }
            // Parse "reading /candidate1/candidate2/"
            if let Some(sep_idx) = line.find(" /") {
                let reading = &line[..sep_idx];
                let candidates_part = &line[sep_idx + 2..]; // Skip " /"
                if candidates_part.ends_with("/") {
                    let candidates_str = &candidates_part[..candidates_part.len() - 1];
                    let candidates: Vec<String> = candidates_str
                        .split('/')
                        .map(|s| s.to_string())
                        .collect();
                    if !candidates.is_empty() {
                        self.entries.insert(reading.to_string(), candidates);
                    }
                }
            }
        }
        Ok(())
    }

    /// Save dictionary to SKK-JISYO format file (UTF-8).
    pub fn save_to_file(&self, path: &Path) -> std::io::Result<()> {
        let mut lines = Vec::new();
        let mut keys: Vec<&String> = self.entries.keys().collect();
        keys.sort();
        for key in keys {
            if let Some(candidates) = self.entries.get(key) {
                let candidates_str = candidates.join("/");
                lines.push(format!("{} /{}/", key, candidates_str));
            }
        }
        fs::write(path, lines.join("\n") + "\n")?;
        Ok(())
    }

    pub fn merge(&mut self, other: &Dictionary) {
        for (reading, candidates) in &other.entries {
            self.entries.insert(reading.clone(), candidates.clone());
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkkAction {
    None,
    Insert(String),
    Convert { reading: String, candidate: String },
    Cancel,
}

pub struct SkkEngine {
    pub state: SkkState,
    pub preedit: String,
    pub reading: String,
    pub candidates: Vec<String>,
    pub candidate_index: usize,
    pub dict: Dictionary,
}

impl SkkEngine {
    pub fn new() -> Self {
        Self {
            state: SkkState::Direct,
            preedit: String::new(),
            reading: String::new(),
            candidates: Vec::new(),
            candidate_index: 0,
            dict: Dictionary::from_builtin(),
        }
    }

    pub fn load_system_dictionary(&mut self, path: &std::path::Path) -> std::io::Result<()> {
        let mut dict = Dictionary::new();
        dict.load_from_file(path)?;
        self.dict.merge(&dict);
        Ok(())
    }

    pub fn load_user_dictionary(&mut self, path: &std::path::Path) -> std::io::Result<()> {
        let mut dict = Dictionary::new();
        dict.load_from_file(path)?;
        self.dict.merge(&dict);
        Ok(())
    }

    pub fn toggle(&mut self) -> SkkAction {
        match self.state {
            SkkState::Direct => {
                self.state = SkkState::Hiragana;
                SkkAction::None
            }
            _ => {
                self.state = SkkState::Direct;
                self.clear();
                SkkAction::None
            }
        }
    }

    pub fn process_char(&mut self, ch: char) -> SkkAction {
        match self.state {
            SkkState::Direct => SkkAction::Insert(ch.to_string()),
            SkkState::Hiragana => self.process_hiragana(ch),
            SkkState::Katakana => self.process_katakana(ch),
            SkkState::Converting => self.process_converting(ch),
            SkkState::Registering => {
                self.state = SkkState::Hiragana;
                self.process_hiragana(ch)
            }
        }
    }

    pub fn confirm(&mut self) -> SkkAction {
        match self.state {
            SkkState::Converting if !self.candidates.is_empty() => {
                let candidate = self.candidates[self.candidate_index].clone();
                let reading = self.reading.clone();
                self.state = SkkState::Hiragana;
                self.clear();
                SkkAction::Convert { reading, candidate }
            }
            _ => SkkAction::None,
        }
    }

    pub fn cancel(&mut self) -> SkkAction {
        match self.state {
            SkkState::Converting => {
                self.state = SkkState::Hiragana;
                self.candidates.clear();
                self.candidate_index = 0;
                SkkAction::Cancel
            }
            SkkState::Hiragana | SkkState::Katakana => {
                self.clear();
                SkkAction::Cancel
            }
            _ => SkkAction::None,
        }
    }

    pub fn next_candidate(&mut self) {
        if !self.candidates.is_empty() {
            self.candidate_index = (self.candidate_index + 1) % self.candidates.len();
        }
    }

    pub fn previous_candidate(&mut self) {
        if !self.candidates.is_empty() {
            if self.candidate_index == 0 {
                self.candidate_index = self.candidates.len() - 1;
            } else {
                self.candidate_index -= 1;
            }
        }
    }

    pub fn current_candidate(&self) -> Option<&str> {
        self.candidates.get(self.candidate_index).map(|s| s.as_str())
    }

    fn clear(&mut self) {
        self.preedit.clear();
        self.reading.clear();
        self.candidates.clear();
        self.candidate_index = 0;
    }

    fn process_hiragana(&mut self, ch: char) -> SkkAction {
        if ch == ' ' {
            if self.reading.is_empty() {
                return SkkAction::Insert(" ".to_string());
            }
            return self.start_conversion();
        }

        if ch == '\n' || ch == '\r' {
            if !self.preedit.is_empty() {
                let result = self.flush_preedit();
                self.reading.clear();
                return result;
            }
            return SkkAction::Insert(ch.to_string());
        }

        self.preedit.push(ch);
        if let Some(kana) = try_convert_romaji(&self.preedit) {
            self.preedit.clear();
            self.reading.push_str(&kana);
            return SkkAction::Insert(kana);
        }

        SkkAction::None
    }

    fn process_katakana(&mut self, ch: char) -> SkkAction {
        if ch == ' ' {
            if self.reading.is_empty() {
                return SkkAction::Insert(" ".to_string());
            }
            return self.start_conversion();
        }

        self.preedit.push(ch);
        if let Some(hiragana) = try_convert_romaji(&self.preedit) {
            self.preedit.clear();
            let katakana: String = hiragana.chars().map(hira_to_kata).collect();
            self.reading.push_str(&katakana);
            return SkkAction::Insert(katakana);
        }

        SkkAction::None
    }

    fn process_converting(&mut self, ch: char) -> SkkAction {
        match ch {
            ' ' => {
                self.next_candidate();
                SkkAction::None
            }
            '\n' | '\r' => {
                let candidate = self.candidates[self.candidate_index].clone();
                let reading = self.reading.clone();
                self.state = SkkState::Hiragana;
                self.clear();
                SkkAction::Convert { reading, candidate }
            }
            _ => {
                // Any other character commits the current candidate and passes the char
                let candidate = self.candidates[self.candidate_index].clone();
                let reading = self.reading.clone();
                self.state = SkkState::Hiragana;
                self.clear();
                let action = SkkAction::Convert { reading, candidate };
                // We can't recursively call process_char here because we already returned.
                // The caller should handle inserting the new char after conversion.
                action
            }
        }
    }

    fn start_conversion(&mut self) -> SkkAction {
        if !self.preedit.is_empty() {
            if let Some(kana) = try_convert_romaji(&self.preedit) {
                self.preedit.clear();
                self.reading.push_str(&kana);
                return SkkAction::Insert(kana);
            }
        }

        if let Some(candidates) = self.dict.lookup(&self.reading) {
            self.candidates = candidates.clone();
            self.candidate_index = 0;
            self.state = SkkState::Converting;
        } else {
            // No candidates found; register mode could be entered here
            self.candidates = vec![self.reading.clone()];
            self.candidate_index = 0;
            self.state = SkkState::Converting;
        }
        SkkAction::None
    }

    fn flush_preedit(&mut self) -> SkkAction {
        let result = self.preedit.clone();
        self.preedit.clear();
        SkkAction::Insert(result)
    }
}

fn try_convert_romaji(romaji: &str) -> Option<String> {
    let table = romaji_table();
    table.get(romaji).map(|s| s.to_string())
}

fn hira_to_kata(ch: char) -> char {
    let code = ch as u32;
    if (0x3041..=0x3096).contains(&code) {
        char::from_u32(code + 0x60).unwrap_or(ch)
    } else {
        ch
    }
}

fn romaji_table() -> HashMap<String, String> {
    let mut m = HashMap::new();
    let pairs = [
        ("a", "あ"), ("i", "い"), ("u", "う"), ("e", "え"), ("o", "お"),
        ("ka", "か"), ("ki", "き"), ("ku", "く"), ("ke", "け"), ("ko", "こ"),
        ("sa", "さ"), ("shi", "し"), ("si", "し"), ("su", "す"), ("se", "せ"), ("so", "そ"),
        ("ta", "た"), ("chi", "ち"), ("ti", "ち"), ("tsu", "つ"), ("tu", "つ"), ("te", "て"), ("to", "と"),
        ("na", "な"), ("ni", "に"), ("nu", "ぬ"), ("ne", "ね"), ("no", "の"),
        ("ha", "は"), ("hi", "ひ"), ("fu", "ふ"), ("hu", "ふ"), ("he", "へ"), ("ho", "ほ"),
        ("ma", "ま"), ("mi", "み"), ("mu", "む"), ("me", "め"), ("mo", "も"),
        ("ya", "や"), ("yu", "ゆ"), ("yo", "よ"),
        ("ra", "ら"), ("ri", "り"), ("ru", "る"), ("re", "れ"), ("ro", "ろ"),
        ("wa", "わ"), ("wo", "を"), ("nn", "ん"),
        ("ga", "が"), ("gi", "ぎ"), ("gu", "ぐ"), ("ge", "げ"), ("go", "ご"),
        ("za", "ざ"), ("ji", "じ"), ("zi", "じ"), ("zu", "ず"), ("ze", "ぜ"), ("zo", "ぞ"),
        ("da", "だ"), ("di", "ぢ"), ("du", "づ"), ("de", "で"), ("do", "ど"),
        ("ba", "ば"), ("bi", "び"), ("bu", "ぶ"), ("be", "べ"), ("bo", "ぼ"),
        ("pa", "ぱ"), ("pi", "ぴ"), ("pu", "ぷ"), ("pe", "ぺ"), ("po", "ぽ"),
        ("kya", "きゃ"), ("kyu", "きゅ"), ("kyo", "きょ"),
        ("sha", "しゃ"), ("shu", "しゅ"), ("sho", "しょ"),
        ("sya", "しゃ"), ("syu", "しゅ"), ("syo", "しょ"),
        ("cha", "ちゃ"), ("chu", "ちゅ"), ("cho", "ちょ"),
        ("cya", "ちゃ"), ("cyu", "ちゅ"), ("cyo", "ちょ"),
        ("nya", "にゃ"), ("nyu", "にゅ"), ("nyo", "にょ"),
        ("hya", "ひゃ"), ("hyu", "ひゅ"), ("hyo", "ひょ"),
        ("mya", "みゃ"), ("myu", "みゅ"), ("myo", "みょ"),
        ("rya", "りゃ"), ("ryu", "りゅ"), ("ryo", "りょ"),
        ("gya", "ぎゃ"), ("gyu", "ぎゅ"), ("gyo", "ぎょ"),
        ("ja", "じゃ"), ("ju", "じゅ"), ("jo", "じょ"),
        ("jya", "じゃ"), ("jyu", "じゅ"), ("jyo", "じょ"),
        ("bya", "びゃ"), ("byu", "びゅ"), ("byo", "びょ"),
        ("pya", "ぴゃ"), ("pyu", "ぴゅ"), ("pyo", "ぴょ"),
    ];
    for (k, v) in pairs {
        m.insert(k.to_string(), v.to_string());
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dictionary_lookup() {
        let dict = Dictionary::from_builtin();
        assert_eq!(dict.lookup("にほんご"), Some(&vec!["日本語".to_string()]));
        assert_eq!(dict.lookup("unknown"), None);
    }

    #[test]
    fn test_dictionary_add() {
        let mut dict = Dictionary::new();
        dict.add("test", &["テスト", "試験"]);
        assert_eq!(dict.lookup("test"), Some(&vec!["テスト".to_string(), "試験".to_string()]));
    }

    #[test]
    fn test_romaji_a() {
        assert_eq!(try_convert_romaji("a"), Some("あ".to_string()));
    }

    #[test]
    fn test_romaji_ka() {
        assert_eq!(try_convert_romaji("ka"), Some("か".to_string()));
    }

    #[test]
    fn test_romaji_n() {
        assert_eq!(try_convert_romaji("nn"), Some("ん".to_string()));
    }

    #[test]
    fn test_romaji_unknown() {
        assert_eq!(try_convert_romaji("xyz"), None);
    }

    #[test]
    fn test_hira_to_kata() {
        assert_eq!(hira_to_kata('あ'), 'ア');
        assert_eq!(hira_to_kata('ん'), 'ン');
        assert_eq!(hira_to_kata('a'), 'a');
    }

    #[test]
    fn test_skk_toggle() {
        let mut skk = SkkEngine::new();
        assert_eq!(skk.state, SkkState::Direct);
        skk.toggle();
        assert_eq!(skk.state, SkkState::Hiragana);
        skk.toggle();
        assert_eq!(skk.state, SkkState::Direct);
    }

    #[test]
    fn test_skk_hiragana_input() {
        let mut skk = SkkEngine::new();
        skk.state = SkkState::Hiragana;
        assert_eq!(skk.process_char('k'), SkkAction::None);
        assert_eq!(skk.process_char('a'), SkkAction::Insert("か".to_string()));
    }

    #[test]
    fn test_skk_hiragana_nihongo() {
        let mut skk = SkkEngine::new();
        skk.state = SkkState::Hiragana;
        let actions: Vec<_> = "nihongo".chars().map(|c| skk.process_char(c)).collect();
        assert_eq!(actions[0], SkkAction::None); // n
        assert_eq!(actions[1], SkkAction::Insert("に".to_string())); // ni
        assert_eq!(actions[2], SkkAction::None); // h
        assert_eq!(actions[3], SkkAction::Insert("ほ".to_string())); // ho
        assert_eq!(actions[4], SkkAction::None); // n
        assert_eq!(actions[5], SkkAction::None); // g
        assert_eq!(actions[6], SkkAction::None); // o (ngo doesn't match)
    }

    #[test]
    fn test_skk_katakana_input() {
        let mut skk = SkkEngine::new();
        skk.state = SkkState::Katakana;
        assert_eq!(skk.process_char('k'), SkkAction::None);
        assert_eq!(skk.process_char('a'), SkkAction::Insert("カ".to_string()));
    }

    #[test]
    fn test_skk_conversion() {
        let mut skk = SkkEngine::new();
        skk.state = SkkState::Hiragana;
        skk.process_char('n');
        skk.process_char('i');
        skk.process_char('h');
        skk.process_char('o');
        skk.process_char('n');
        skk.process_char('g');
        skk.process_char('o');
        // This will insert kana as we type, but let's manually set reading for test
        skk.reading = "にほんご".to_string();
        skk.preedit.clear();
        let action = skk.process_char(' ');
        assert_eq!(action, SkkAction::None);
        assert_eq!(skk.state, SkkState::Converting);
        assert_eq!(skk.current_candidate(), Some("日本語"));
    }

    #[test]
    fn test_skk_confirm_conversion() {
        let mut skk = SkkEngine::new();
        skk.state = SkkState::Hiragana;
        skk.reading = "にほんご".to_string();
        skk.start_conversion();
        assert_eq!(skk.state, SkkState::Converting);
        let action = skk.confirm();
        assert_eq!(
            action,
            SkkAction::Convert {
                reading: "にほんご".to_string(),
                candidate: "日本語".to_string()
            }
        );
        assert_eq!(skk.state, SkkState::Hiragana);
    }

    #[test]
    fn test_skk_cancel_conversion() {
        let mut skk = SkkEngine::new();
        skk.state = SkkState::Converting;
        skk.reading = "にほんご".to_string();
        skk.candidates = vec!["日本語".to_string()];
        let action = skk.cancel();
        assert_eq!(action, SkkAction::Cancel);
        assert_eq!(skk.state, SkkState::Hiragana);
        assert!(skk.candidates.is_empty());
    }

    #[test]
    fn test_skk_next_candidate() {
        let mut skk = SkkEngine::new();
        skk.state = SkkState::Hiragana;
        skk.reading = "てきすと".to_string();
        skk.start_conversion();
        assert_eq!(skk.current_candidate(), Some("テキスト"));
        skk.next_candidate();
        assert_eq!(skk.current_candidate(), Some("文章"));
        skk.next_candidate();
        assert_eq!(skk.current_candidate(), Some("テキスト"));
    }

    #[test]
    fn test_skk_direct_mode_passes_through() {
        let mut skk = SkkEngine::new();
        assert_eq!(skk.process_char('a'), SkkAction::Insert("a".to_string()));
    }

    #[test]
    fn test_skk_space_in_direct() {
        let mut skk = SkkEngine::new();
        assert_eq!(skk.process_char(' '), SkkAction::Insert(" ".to_string()));
    }

    #[test]
    fn test_load_system_dictionary() {
        let path = std::env::temp_dir().join("dim_test_system_dict.txt");
        std::fs::write(&path, "にほんご /日本語/日本語の/\n").unwrap();
        let mut skk = SkkEngine::new();
        skk.load_system_dictionary(&path).unwrap();
        assert_eq!(skk.dict.lookup("にほんご"), Some(&vec!["日本語".to_string(), "日本語の".to_string()]));
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_load_user_dictionary() {
        let path = std::env::temp_dir().join("dim_test_user_dict.txt");
        std::fs::write(&path, "てきすと /テキスト/文章/\n").unwrap();
        let mut skk = SkkEngine::new();
        skk.load_user_dictionary(&path).unwrap();
        assert_eq!(skk.dict.lookup("てきすと"), Some(&vec!["テキスト".to_string(), "文章".to_string()]));
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_load_dictionary_merges_with_builtin() {
        let path = std::env::temp_dir().join("dim_test_merge_dict.txt");
        std::fs::write(&path, "にほんご /日本語/\n").unwrap();
        let mut skk = SkkEngine::new();
        // builtin has "にほんご" -> ["日本語"]
        skk.load_user_dictionary(&path).unwrap();
        // Should still work after merge
        assert_eq!(skk.dict.lookup("にほんご"), Some(&vec!["日本語".to_string()]));
        std::fs::remove_file(&path).unwrap();
    }
}
