use regex::Regex;
use std::collections::HashMap;

pub mod casing;
pub mod patterns;

pub use casing::*;
pub use patterns::*;

#[derive(Debug, Clone, Default)]
pub struct VocabularyEntry {
    pub spoken: String,
    pub written: String,
}

impl VocabularyEntry {
    pub fn new(spoken: impl Into<String>, written: impl Into<String>) -> Self {
        Self {
            spoken: spoken.into(),
            written: written.into(),
        }
    }
}

pub struct PostProcessor {
    keywords: HashMap<String, String>,
    file_extensions: Vec<&'static str>,
    vocabulary: Vec<(Regex, String)>,
}

impl PostProcessor {
    pub fn new() -> Self {
        let mut keywords = HashMap::new();

        let kw_list = [
            "if",
            "else",
            "for",
            "while",
            "do",
            "switch",
            "case",
            "break",
            "continue",
            "return",
            "function",
            "const",
            "let",
            "var",
            "class",
            "struct",
            "enum",
            "interface",
            "type",
            "import",
            "export",
            "from",
            "as",
            "default",
            "async",
            "await",
            "try",
            "catch",
            "finally",
            "throw",
            "new",
            "this",
            "self",
            "super",
            "public",
            "private",
            "protected",
            "static",
            "final",
            "abstract",
            "virtual",
            "override",
            "implements",
            "extends",
            "null",
            "undefined",
            "none",
            "nil",
            "true",
            "false",
            "and",
            "or",
            "not",
            "in",
            "is",
            "typeof",
            "instanceof",
            "void",
            "int",
            "float",
            "double",
            "string",
            "bool",
            "boolean",
            "char",
            "array",
            "list",
            "map",
            "set",
            "dict",
            "tuple",
            "option",
            "result",
            "println",
            "print",
            "console",
            "log",
            "debug",
            "info",
            "warn",
            "error",
        ];

        for kw in kw_list {
            keywords.insert(kw.to_lowercase(), kw.to_string());
        }

        Self {
            keywords,
            file_extensions: vec![
                "js", "ts", "tsx", "jsx", "rs", "py", "go", "rb", "java", "cpp", "c", "h", "hpp",
                "css", "scss", "sass", "less", "html", "htm", "json", "yaml", "yml", "toml", "xml",
                "md", "txt", "sh", "bash", "zsh", "fish", "sql", "vue", "svelte", "astro", "php",
                "swift", "kt", "scala", "ex", "exs", "erl", "hs", "ml", "fs", "clj", "lisp", "r",
                "jl", "lua", "pl", "pm",
            ],
            vocabulary: Vec::new(),
        }
    }

    pub fn with_vocabulary(entries: &[VocabularyEntry]) -> Self {
        let mut pp = Self::new();
        pp.set_vocabulary(entries);
        pp
    }

    pub fn set_vocabulary(&mut self, entries: &[VocabularyEntry]) {
        let mut compiled: Vec<(Regex, String)> = entries
            .iter()
            .filter(|e| !e.spoken.trim().is_empty() && !e.written.is_empty())
            .filter_map(|e| {
                let escaped = regex::escape(e.spoken.trim());
                let pattern = format!(r"(?i)\b{}\b", escaped);
                Regex::new(&pattern).ok().map(|re| (re, e.written.clone()))
            })
            .collect();

        compiled.sort_by(|a, b| b.0.as_str().len().cmp(&a.0.as_str().len()));
        self.vocabulary = compiled;
    }

    fn apply_vocabulary(&self, text: &str) -> (String, Vec<(String, String)>) {
        if self.vocabulary.is_empty() {
            return (text.to_string(), Vec::new());
        }
        let mut result = text.to_string();
        let mut restore_map: Vec<(String, String)> = Vec::new();
        for (i, (pattern, replacement)) in self.vocabulary.iter().enumerate() {
            let placeholder = format!("\u{E000}VOCAB{}\u{E000}", i);
            restore_map.push((placeholder.clone(), replacement.clone()));
            result = pattern
                .replace_all(&result, placeholder.as_str())
                .to_string();
        }
        (result, restore_map)
    }

    pub fn process(&self, text: &str) -> String {
        let mut result = text.to_string();

        result = self.process_voice_commands(&result);

        let (vocab_result, restore_map) = self.apply_vocabulary(&result);
        result = vocab_result;

        result = self.process_explicit_casing(&result);
        result = self.process_functions(&result);
        result = self.process_file_mentions(&result);
        result = self.process_file_paths(&result);

        result = self.process_variables(&result);
        result = self.process_classes(&result);
        result = self.process_symbols(&result);

        result = self.fix_sentence_casing(&result);

        result = self.process_abbreviations(&result);
        result = self.process_keywords(&result);
        result = self.cleanup_whitespace(&result);

        for (placeholder, replacement) in &restore_map {
            result = result.replace(placeholder, replacement);
        }

        result
    }

    pub fn extract_voice_commands(&self, text: &str) -> String {
        let mut result = self.process_voice_commands(text);
        let (vocab_result, restore_map) = self.apply_vocabulary(&result);
        result = vocab_result;
        for (placeholder, replacement) in &restore_map {
            result = result.replace(placeholder, replacement);
        }
        result
    }

    fn process_voice_commands(&self, text: &str) -> String {
        let mut result = text.to_string();

        result = COMMAND_ALL_CAPS
            .replace_all(&result, |caps: &regex::Captures| caps[1].to_uppercase())
            .to_string();

        result = COMMAND_NO_CAPS
            .replace_all(&result, |caps: &regex::Captures| caps[1].to_lowercase())
            .to_string();

        result = COMMAND_CAP
            .replace_all(&result, |caps: &regex::Captures| {
                let word = &caps[1];
                let mut chars = word.chars();
                match chars.next() {
                    Some(c) => format!("{}{}", c.to_uppercase(), chars.as_str()),
                    None => String::new(),
                }
            })
            .to_string();

        result = NEW_PARAGRAPH_PATTERN
            .replace_all(&result, "\n\n")
            .to_string();
        result = NEWLINE_PATTERN.replace_all(&result, "\n").to_string();

        result = COMMAND_ELLIPSIS.replace_all(&result, "...").to_string();
        result = COMMAND_QUESTION_MARK.replace_all(&result, "?").to_string();
        result = COMMAND_EXCLAMATION.replace_all(&result, "!").to_string();
        result = COMMAND_OPEN_QUOTE.replace_all(&result, "\"").to_string();
        result = COMMAND_CLOSE_QUOTE.replace_all(&result, "\"").to_string();
        result = COMMAND_SINGLE_QUOTE.replace_all(&result, "'").to_string();
        result = COMMAND_QUOTE.replace_all(&result, "\"").to_string();
        result = COMMAND_APOSTROPHE.replace_all(&result, "'").to_string();
        result = COMMAND_COMMA.replace_all(&result, ",").to_string();
        result = COMMAND_PERIOD.replace_all(&result, ".").to_string();

        result = COMMAND_AMPERSAND.replace_all(&result, "&").to_string();
        result = COMMAND_AT_SIGN.replace_all(&result, "@").to_string();
        result = COMMAND_HASH.replace_all(&result, "#").to_string();
        result = COMMAND_PERCENT.replace_all(&result, "%").to_string();
        result = COMMAND_DOLLAR.replace_all(&result, "$").to_string();
        result = COMMAND_ASTERISK.replace_all(&result, "*").to_string();
        result = COMMAND_PLUS.replace_all(&result, "+").to_string();
        result = COMMAND_MINUS.replace_all(&result, "-").to_string();
        result = COMMAND_TILDE.replace_all(&result, "~").to_string();
        result = COMMAND_CARET.replace_all(&result, "^").to_string();
        result = COMMAND_PIPE.replace_all(&result, "|").to_string();
        result = COMMAND_LESS_THAN.replace_all(&result, "<").to_string();
        result = COMMAND_GREATER_THAN.replace_all(&result, ">").to_string();

        result = COMMAND_NO_SPACE.replace_all(&result, "").to_string();
        result = COMMAND_SPACE.replace_all(&result, " ").to_string();

        result = COMMAND_DELETE_THAT
            .replace_all(&result, "[[DELETE_LAST]]")
            .to_string();
        result = COMMAND_UNDO.replace_all(&result, "[[UNDO]]").to_string();
        result = COMMAND_REDO.replace_all(&result, "[[REDO]]").to_string();
        result = COMMAND_SELECT_ALL
            .replace_all(&result, "[[SELECT_ALL]]")
            .to_string();
        result = COMMAND_COPY_THAT
            .replace_all(&result, "[[COPY]]")
            .to_string();
        result = COMMAND_CUT_THAT.replace_all(&result, "[[CUT]]").to_string();
        result = COMMAND_PASTE_THAT
            .replace_all(&result, "[[PASTE]]")
            .to_string();

        result = COMMAND_BACKSPACE
            .replace_all(&result, "[[BACKSPACE]]")
            .to_string();
        result = COMMAND_DELETE_FORWARD
            .replace_all(&result, "[[DELETE_FORWARD]]")
            .to_string();
        result = COMMAND_DELETE_WORD
            .replace_all(&result, "[[DELETE_WORD]]")
            .to_string();
        result = COMMAND_DELETE_LINE
            .replace_all(&result, "[[DELETE_LINE]]")
            .to_string();
        result = COMMAND_ENTER.replace_all(&result, "[[ENTER]]").to_string();
        result = COMMAND_TAB_KEY.replace_all(&result, "[[TAB]]").to_string();
        result = COMMAND_ESCAPE
            .replace_all(&result, "[[ESCAPE]]")
            .to_string();
        result = COMMAND_PAGE_UP
            .replace_all(&result, "[[PAGE_UP]]")
            .to_string();
        result = COMMAND_PAGE_DOWN
            .replace_all(&result, "[[PAGE_DOWN]]")
            .to_string();

        result = COMMAND_GO_LEFT.replace_all(&result, "[[LEFT]]").to_string();
        result = COMMAND_GO_RIGHT
            .replace_all(&result, "[[RIGHT]]")
            .to_string();
        result = COMMAND_GO_UP.replace_all(&result, "[[UP]]").to_string();
        result = COMMAND_GO_DOWN.replace_all(&result, "[[DOWN]]").to_string();
        result = COMMAND_SELECT_LEFT
            .replace_all(&result, "[[SELECT_LEFT]]")
            .to_string();
        result = COMMAND_SELECT_RIGHT
            .replace_all(&result, "[[SELECT_RIGHT]]")
            .to_string();
        result = COMMAND_SELECT_UP
            .replace_all(&result, "[[SELECT_UP]]")
            .to_string();
        result = COMMAND_SELECT_DOWN
            .replace_all(&result, "[[SELECT_DOWN]]")
            .to_string();
        result = COMMAND_SELECT_WORD_LEFT
            .replace_all(&result, "[[SELECT_WORD_LEFT]]")
            .to_string();
        result = COMMAND_SELECT_WORD_RIGHT
            .replace_all(&result, "[[SELECT_WORD_RIGHT]]")
            .to_string();
        result = COMMAND_GO_WORD_LEFT
            .replace_all(&result, "[[WORD_LEFT]]")
            .to_string();
        result = COMMAND_GO_WORD_RIGHT
            .replace_all(&result, "[[WORD_RIGHT]]")
            .to_string();
        result = COMMAND_SELECT_TO_START
            .replace_all(&result, "[[SELECT_TO_START]]")
            .to_string();
        result = COMMAND_SELECT_TO_END
            .replace_all(&result, "[[SELECT_TO_END]]")
            .to_string();
        result = COMMAND_GO_START
            .replace_all(&result, "[[HOME]]")
            .to_string();
        result = COMMAND_GO_END.replace_all(&result, "[[END]]").to_string();

        result
    }

    fn process_explicit_casing(&self, text: &str) -> String {
        let mut result = text.to_string();

        result = CAMEL_CASE_PATTERN
            .replace_all(&result, |caps: &regex::Captures| to_camel_case(&caps[1]))
            .to_string();

        result = SNAKE_CASE_PATTERN
            .replace_all(&result, |caps: &regex::Captures| to_snake_case(&caps[1]))
            .to_string();

        result = PASCAL_CASE_PATTERN
            .replace_all(&result, |caps: &regex::Captures| to_pascal_case(&caps[1]))
            .to_string();

        result = KEBAB_CASE_PATTERN
            .replace_all(&result, |caps: &regex::Captures| to_kebab_case(&caps[1]))
            .to_string();

        result = CONSTANT_CASE_PATTERN
            .replace_all(&result, |caps: &regex::Captures| to_constant_case(&caps[2]))
            .to_string();

        result
    }

    fn process_functions(&self, text: &str) -> String {
        FUNCTION_PATTERN
            .replace_all(text, |caps: &regex::Captures| {
                let name = to_camel_case(&caps[2]);
                format!("{}()", name)
            })
            .to_string()
    }

    fn process_file_mentions(&self, text: &str) -> String {
        let mut result = text.to_string();

        result = FILE_MENTION_PATTERN
            .replace_all(&result, |caps: &regex::Captures| {
                let preposition = &caps[1];
                let filename = caps[2].to_lowercase();
                let ext = caps[3].to_lowercase();
                format!("{} @{}.{}", preposition, filename, ext)
            })
            .to_string();

        result = STANDALONE_FILE_MENTION_PATTERN
            .replace_all(&result, |caps: &regex::Captures| {
                let filename = caps[1].to_lowercase();
                let ext = caps[2].to_lowercase();
                format!("@{}.{}", filename, ext)
            })
            .to_string();

        result
    }

    fn process_file_paths(&self, text: &str) -> String {
        let mut result = FILE_PATH_PATTERN
            .replace_all(text, |caps: &regex::Captures| {
                format!("@{}.{}", caps[1].to_lowercase(), caps[2].to_lowercase())
            })
            .to_string();

        result = FILE_WITH_PERIOD_PATTERN
            .replace_all(&result, |caps: &regex::Captures| {
                let filename = &caps[1];
                let ext = caps[2].to_lowercase();

                if FILE_EXTENSIONS.contains(&ext.as_str()) {
                    format!("@{}.{}", filename.to_lowercase(), ext)
                } else {
                    format!("{}.{}", filename, &caps[2])
                }
            })
            .to_string();

        result = result.replace("@@", "@");

        result
    }

    fn process_variables(&self, text: &str) -> String {
        VARIABLE_PATTERN
            .replace_all(text, |caps: &regex::Captures| {
                let keyword = caps[1].to_lowercase();
                let name = to_camel_case(&caps[2]);
                format!("{} {}", keyword, name)
            })
            .to_string()
    }

    fn process_classes(&self, text: &str) -> String {
        CLASS_PATTERN
            .replace_all(text, |caps: &regex::Captures| {
                let name = to_pascal_case(&caps[1]);
                format!("class {}", name)
            })
            .to_string()
    }

    fn process_symbols(&self, text: &str) -> String {
        let mut result = text.to_string();

        result = SEMICOLON_PATTERN.replace_all(&result, ";").to_string();
        result = BACKSLASH_PATTERN.replace_all(&result, "\\").to_string();
        result = SLASH_PATTERN.replace_all(&result, "/").to_string();
        result = UNDERSCORE_PATTERN.replace_all(&result, "_").to_string();
        result = HYPHEN_PATTERN.replace_all(&result, "-").to_string();
        result = COLON_PATTERN.replace_all(&result, ":").to_string();
        result = ARROW_PATTERN.replace_all(&result, "=>").to_string();
        result = EQUALS_PATTERN.replace_all(&result, "=").to_string();

        result = OPEN_PAREN_PATTERN.replace_all(&result, "(").to_string();
        result = CLOSE_PAREN_PATTERN.replace_all(&result, ")").to_string();
        result = OPEN_BRACE_PATTERN.replace_all(&result, "{").to_string();
        result = CLOSE_BRACE_PATTERN.replace_all(&result, "}").to_string();
        result = OPEN_SQUARE_PATTERN.replace_all(&result, "[").to_string();
        result = CLOSE_SQUARE_PATTERN.replace_all(&result, "]").to_string();

        result = TAB_PATTERN.replace_all(&result, "\t").to_string();

        result = self.process_standalone_dots(&result);

        result
    }

    fn process_standalone_dots(&self, text: &str) -> String {
        let mut lines_result = Vec::new();

        for line in text.split('\n') {
            let mut line_result = String::new();
            let words: Vec<&str> = line.split_whitespace().collect();

            for (i, word) in words.iter().enumerate() {
                if i > 0 {
                    line_result.push(' ');
                }

                let lower = word.to_lowercase();
                if lower == "dot" || lower == "period" {
                    let next_is_ext = words
                        .get(i + 1)
                        .map(|w| self.file_extensions.contains(&w.to_lowercase().as_str()))
                        .unwrap_or(false);

                    if next_is_ext {
                        line_result.push_str(word);
                    } else {
                        line_result.push('.');
                    }
                } else {
                    line_result.push_str(word);
                }
            }

            lines_result.push(line_result);
        }

        lines_result.join("\n")
    }

    fn process_abbreviations(&self, text: &str) -> String {
        let mut result = String::new();
        let mut last_end = 0;

        for cap in ABBREV_PATTERN.captures_iter(text) {
            let m = cap.get(1).unwrap();
            let start = m.start();

            let is_file_ext = if start > 0 {
                let prev_char = text.chars().nth(start - 1).unwrap_or(' ');
                prev_char == '@' || prev_char == '.'
            } else {
                false
            };

            result.push_str(&text[last_end..start]);

            if is_file_ext {
                result.push_str(&m.as_str().to_lowercase());
            } else {
                result.push_str(&m.as_str().to_uppercase());
            }

            last_end = m.end();
        }

        result.push_str(&text[last_end..]);
        result
    }

    fn process_keywords(&self, text: &str) -> String {
        let mut result = String::new();
        let mut last_end = 0;

        for (i, c) in text.char_indices() {
            if c.is_alphabetic()
                && (i == 0
                    || !text
                        .chars()
                        .nth(i - 1)
                        .map(|p| p.is_alphanumeric())
                        .unwrap_or(false))
            {
                let word_start = i;
                let word_end = text[i..]
                    .char_indices()
                    .find(|(_, c)| !c.is_alphanumeric())
                    .map(|(j, _)| i + j)
                    .unwrap_or(text.len());

                let word = &text[word_start..word_end];
                let lower = word.to_lowercase();

                if let Some(proper_case) = self.keywords.get(&lower) {
                    result.push_str(&text[last_end..word_start]);
                    result.push_str(proper_case);
                    last_end = word_end;
                }
            }
        }

        result.push_str(&text[last_end..]);

        if result.is_empty() {
            text.to_string()
        } else {
            result
        }
    }

    fn fix_sentence_casing(&self, text: &str) -> String {
        let mut result = String::new();
        let mut capitalize_next = true;
        let chars: Vec<char> = text.chars().collect();

        for (i, &c) in chars.iter().enumerate() {
            if capitalize_next && c.is_alphabetic() {
                result.push(c.to_uppercase().next().unwrap_or(c));
                capitalize_next = false;
            } else {
                result.push(c);
                if c == '.' || c == '!' || c == '?' {
                    let before_is_alnum = i > 0
                        && chars
                            .get(i - 1)
                            .map(|ch| ch.is_alphanumeric())
                            .unwrap_or(false);
                    let after_is_alnum = chars
                        .get(i + 1)
                        .map(|ch| ch.is_alphanumeric())
                        .unwrap_or(false);

                    if c == '.' && before_is_alnum && after_is_alnum {
                        capitalize_next = false;
                    } else {
                        capitalize_next = true;
                    }
                }
            }
        }

        result
    }

    fn cleanup_whitespace(&self, text: &str) -> String {
        let mut result = String::new();
        let mut prev_was_space = false;

        for c in text.chars() {
            if c.is_whitespace() && c != '\n' && c != '\t' {
                if !prev_was_space {
                    result.push(' ');
                    prev_was_space = true;
                }
            } else {
                result.push(c);
                prev_was_space = false;
            }
        }

        result.trim().to_string()
    }
}

impl Default for PostProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camel_case() {
        let pp = PostProcessor::new();
        assert_eq!(pp.process("camel case hello world"), "HelloWorld");
        assert_eq!(
            pp.process("use camel case get user data"),
            "Use getUserData"
        );
    }

    #[test]
    fn test_snake_case() {
        let pp = PostProcessor::new();
        assert_eq!(pp.process("snake case hello world"), "Hello_world");
        assert_eq!(pp.process("use snake case hello world"), "Use hello_world");
    }

    #[test]
    fn test_file_paths() {
        let pp = PostProcessor::new();
        assert_eq!(pp.process("open index dot ts"), "Open @index.ts");
        assert_eq!(pp.process("main dot rs"), "@Main.rs");
    }

    #[test]
    fn test_file_mentions() {
        let pp = PostProcessor::new();
        assert_eq!(
            pp.process("fix bug in index dot ts"),
            "Fix bug in @index.ts"
        );
        assert_eq!(pp.process("check the app dot tsx"), "Check the @app.tsx");
        assert_eq!(pp.process("edit main dot rs"), "Edit @main.rs");
        assert_eq!(pp.process("refactor utils dot py"), "Refactor @utils.py");
    }

    #[test]
    fn test_function() {
        let pp = PostProcessor::new();
        assert_eq!(pp.process("function get user"), "GetUser()");
        assert_eq!(pp.process("call function get user"), "Call getUser()");
    }

    #[test]
    fn test_symbols() {
        let pp = PostProcessor::new();
        let result = pp.process("hello slash world");
        assert!(result.contains("/"));
        let result = pp.process("a equals b");
        assert!(result.contains("="));
    }

    #[test]
    fn test_keywords() {
        let pp = PostProcessor::new();
        let result = pp.process("this is true and false");
        assert!(result.contains("true"), "Expected 'true' in '{}'", result);
        assert!(result.contains("false"), "Expected 'false' in '{}'", result);
    }

    #[test]
    fn test_voice_commands() {
        let pp = PostProcessor::new();

        let result = pp.process("hello new line world");
        assert!(
            result.contains('\n'),
            "Expected newline character in '{}'",
            result
        );

        let result = pp.process("hello new paragraph world");
        assert!(
            result.contains("\n\n"),
            "Expected double newline in '{}'",
            result
        );
    }

    #[test]
    fn test_voice_commands_with_whisper_punctuation() {
        let pp = PostProcessor::new();

        let result = pp.process("say hello world!");
        assert!(result.contains("!"), "Expected '!' in '{}'", result);

        let result = pp.process("type hello world?");
        assert!(result.contains("?"), "Expected '?' in '{}'", result);
    }
}
