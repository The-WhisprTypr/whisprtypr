use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

/// A single user-defined custom vocabulary entry used by the post-processor.
///
/// `spoken` is the phrase the user expects Whisper to output (or the
/// misheard form they want to correct) and `written` is the canonical
/// replacement text that should appear in the final output.
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

/// Post-processor for transcribed text
/// Handles proper casing, file paths, function names, and programming patterns
pub struct PostProcessor {
    /// Common programming keywords that should be lowercase
    keywords: HashMap<String, String>,
    /// File extensions for path detection
    file_extensions: Vec<&'static str>,
    /// Compiled vocabulary replacement regexes, longest-spoken-first so
    /// multi-word terms win over single-word substrings.
    vocabulary: Vec<(regex::Regex, String)>,
}

lazy_static! {
    // Pattern: "function name" or "func name" -> functionName()
    static ref FUNCTION_PATTERN: Regex = Regex::new(
        r"(?i)\b(function|func|method|def)\s+([a-z]+(?:\s+[a-z]+)*)\b"
    ).unwrap();

    // Pattern: "file name dot extension" -> @filename.extension
    // Matches: "build dot rs", "index dot ts", etc.
    static ref FILE_PATH_PATTERN: Regex = Regex::new(
        r"(?i)\b([a-z][a-z0-9_-]*)\s+dot\s+(js|ts|tsx|jsx|rs|py|go|rb|java|cpp|c|h|hpp|css|scss|html|json|yaml|yml|toml|md|txt|sh|bash|sql|vue|svelte|astro)\b"
    ).unwrap();

    // Pattern: "file name.extension" -> @filename.extension (when Whisper outputs actual period)
    // Matches: "build.rs", "index.ts", etc. (adds @ prefix)
    static ref FILE_WITH_PERIOD_PATTERN: Regex = Regex::new(
        r"(?i)\b([a-z][a-z0-9_-]*)\.([a-z]{1,5})\b"
    ).unwrap();

    // Known file extensions for the period pattern
    static ref FILE_EXTENSIONS: Vec<&'static str> = vec![
        "js", "ts", "tsx", "jsx", "rs", "py", "go", "rb", "java", "cpp", "c",
        "h", "hpp", "css", "scss", "html", "json", "yaml", "yml", "toml", "md",
        "txt", "sh", "bash", "sql", "vue", "svelte", "astro", "env", "lock"
    ];

    // Pattern: "slash" -> /
    static ref SLASH_PATTERN: Regex = Regex::new(r"(?i)\b(forward\s+)?slash\b").unwrap();

    // Pattern: "backslash" -> \
    static ref BACKSLASH_PATTERN: Regex = Regex::new(r"(?i)\bback\s*slash\b").unwrap();

    // Pattern: "underscore" -> _
    static ref UNDERSCORE_PATTERN: Regex = Regex::new(r"(?i)\bunderscore\b").unwrap();

    // Pattern: "hyphen" or "dash" -> -
    static ref HYPHEN_PATTERN: Regex = Regex::new(r"(?i)\b(hyphen|dash)\b").unwrap();

    // Pattern: "dot" or "period" (standalone) -> .
    static ref DOT_PATTERN: Regex = Regex::new(r"(?i)\b(dot|period)\b").unwrap();

    // Pattern: "colon" -> :
    static ref COLON_PATTERN: Regex = Regex::new(r"(?i)\bcolon\b").unwrap();

    // Pattern: "semicolon" -> ;
    static ref SEMICOLON_PATTERN: Regex = Regex::new(r"(?i)\bsemi\s*colon\b").unwrap();

    // Pattern: "equals" or "equal sign" -> =
    static ref EQUALS_PATTERN: Regex = Regex::new(r"(?i)\b(equals?(\s+sign)?|equal\s+to)\b").unwrap();

    // Pattern: "arrow" or "fat arrow" -> =>
    static ref ARROW_PATTERN: Regex = Regex::new(r"(?i)\b(fat\s+)?arrow\b").unwrap();

    // Pattern: "open paren" -> (
    static ref OPEN_PAREN_PATTERN: Regex = Regex::new(r"(?i)\bopen\s*(paren|parenthesis|bracket)\b").unwrap();

    // Pattern: "close paren" -> )
    static ref CLOSE_PAREN_PATTERN: Regex = Regex::new(r"(?i)\bclose\s*(paren|parenthesis|bracket)\b").unwrap();

    // Pattern: "open brace" or "open curly" -> {
    static ref OPEN_BRACE_PATTERN: Regex = Regex::new(r"(?i)\bopen\s*(brace|curly)\b").unwrap();

    // Pattern: "close brace" or "close curly" -> }
    static ref CLOSE_BRACE_PATTERN: Regex = Regex::new(r"(?i)\bclose\s*(brace|curly)\b").unwrap();

    // Pattern: "open square" or "open bracket" -> [
    static ref OPEN_SQUARE_PATTERN: Regex = Regex::new(r"(?i)\bopen\s*square(\s*bracket)?\b").unwrap();

    // Pattern: "close square" or "close bracket" -> ]
    static ref CLOSE_SQUARE_PATTERN: Regex = Regex::new(r"(?i)\bclose\s*square(\s*bracket)?\b").unwrap();

    // Pattern: "new line" or "newline" -> \n (with optional trailing punctuation from Whisper)
    static ref NEWLINE_PATTERN: Regex = Regex::new(r"(?i)\bnew\s*line\b[.,!?]?").unwrap();

    // Pattern: "new paragraph" -> \n\n (with optional trailing punctuation from Whisper)
    static ref NEW_PARAGRAPH_PATTERN: Regex = Regex::new(r"(?i)\bnew\s*paragraph\b[.,!?]?").unwrap();

    // Pattern: "tab" -> \t (only when it seems intentional)
    static ref TAB_PATTERN: Regex = Regex::new(r"(?i)\btab\s+(character|key)\b").unwrap();

    // ==================== VOICE COMMANDS ====================

    // Punctuation commands - require "insert" prefix to avoid false positives
    static ref COMMAND_COMMA: Regex = Regex::new(r"(?i)\binsert\s+comma\b").unwrap();
    static ref COMMAND_PERIOD: Regex = Regex::new(r"(?i)\binsert\s+(period|full\s+stop)\b").unwrap();
    static ref COMMAND_QUESTION_MARK: Regex = Regex::new(r"(?i)\binsert\s+question\s*mark\b").unwrap();
    static ref COMMAND_EXCLAMATION: Regex = Regex::new(r"(?i)\binsert\s+(exclamation\s*(mark|point)?|bang)\b").unwrap();
    static ref COMMAND_APOSTROPHE: Regex = Regex::new(r"(?i)\binsert\s+apostrophe\b").unwrap();
    static ref COMMAND_QUOTE: Regex = Regex::new(r"(?i)\binsert\s+(double\s+)?quote\b").unwrap();
    static ref COMMAND_SINGLE_QUOTE: Regex = Regex::new(r"(?i)\binsert\s+single\s+quote\b").unwrap();
    static ref COMMAND_OPEN_QUOTE: Regex = Regex::new(r"(?i)\bopen\s+(double\s+)?quote\b").unwrap();
    static ref COMMAND_CLOSE_QUOTE: Regex = Regex::new(r"(?i)\bclose\s+(double\s+)?quote\b").unwrap();
    static ref COMMAND_ELLIPSIS: Regex = Regex::new(r"(?i)\binsert\s+ellipsis\b").unwrap();
    static ref COMMAND_AMPERSAND: Regex = Regex::new(r"(?i)\binsert\s+ampersand\b").unwrap();
    static ref COMMAND_AT_SIGN: Regex = Regex::new(r"(?i)\binsert\s+at\s*sign\b").unwrap();
    static ref COMMAND_HASH: Regex = Regex::new(r"(?i)\binsert\s+(hash|hashtag|pound\s*sign|number\s*sign)\b").unwrap();
    static ref COMMAND_PERCENT: Regex = Regex::new(r"(?i)\binsert\s+percent(\s*sign)?\b").unwrap();
    static ref COMMAND_DOLLAR: Regex = Regex::new(r"(?i)\binsert\s+dollar(\s*sign)?\b").unwrap();
    static ref COMMAND_ASTERISK: Regex = Regex::new(r"(?i)\binsert\s+(asterisk|star)\b").unwrap();
    static ref COMMAND_PLUS: Regex = Regex::new(r"(?i)\binsert\s+plus(\s*sign)?\b").unwrap();
    static ref COMMAND_MINUS: Regex = Regex::new(r"(?i)\binsert\s+minus(\s*sign)?\b").unwrap();
    static ref COMMAND_TILDE: Regex = Regex::new(r"(?i)\binsert\s+tilde\b").unwrap();
    static ref COMMAND_CARET: Regex = Regex::new(r"(?i)\binsert\s+caret\b").unwrap();
    static ref COMMAND_PIPE: Regex = Regex::new(r"(?i)\binsert\s+(pipe|vertical\s*bar)\b").unwrap();
    static ref COMMAND_LESS_THAN: Regex = Regex::new(r"(?i)\binsert\s+(less\s*than|left\s*angle(\s*bracket)?)\b").unwrap();
    static ref COMMAND_GREATER_THAN: Regex = Regex::new(r"(?i)\binsert\s+(greater\s*than|right\s*angle(\s*bracket)?)\b").unwrap();

    // Special text commands - these are action commands (with optional trailing punctuation from Whisper)
    static ref COMMAND_DELETE_THAT: Regex = Regex::new(r"(?i)\b(delete\s+that|scratch\s+that|remove\s+that|delete\s+last|scratch\s+last)\b[.,!?]?").unwrap();
    static ref COMMAND_UNDO: Regex = Regex::new(r"(?i)\bundo(\s+(that|last|it))?\b[.,!?]?").unwrap();
    static ref COMMAND_REDO: Regex = Regex::new(r"(?i)\bredo(\s+(that|last|it))?\b[.,!?]?").unwrap();
    static ref COMMAND_SELECT_ALL: Regex = Regex::new(r"(?i)\bselect\s+all(\s+text)?\b[.,!?]?").unwrap();
    static ref COMMAND_COPY_THAT: Regex = Regex::new(r"(?i)\bcopy(\s+(that|this|selection|it))?\b[.,!?]?").unwrap();
    static ref COMMAND_CUT_THAT: Regex = Regex::new(r"(?i)\bcut(\s+(that|this|selection|it))?\b[.,!?]?").unwrap();
    static ref COMMAND_PASTE_THAT: Regex = Regex::new(r"(?i)\bpaste(\s+(that|here|it))?\b[.,!?]?").unwrap();

    // Additional navigation/editing commands
    static ref COMMAND_BACKSPACE: Regex = Regex::new(r"(?i)\b(backspace|delete\s+character|remove\s+character)\b[.,!?]?").unwrap();
    static ref COMMAND_DELETE_FORWARD: Regex = Regex::new(r"(?i)\b(forward\s+delete|delete\s+forward|delete\s+next\s+character|delete\s+key)\b[.,!?]?").unwrap();
    static ref COMMAND_DELETE_WORD: Regex = Regex::new(r"(?i)\b(delete\s+word|remove\s+word|backspace\s+word)\b[.,!?]?").unwrap();
    static ref COMMAND_DELETE_LINE: Regex = Regex::new(r"(?i)\b(delete\s+line|remove\s+line|clear\s+line)\b[.,!?]?").unwrap();
    static ref COMMAND_ENTER: Regex = Regex::new(r"(?i)\b(press\s+enter|hit\s+enter|enter(\s+key)?)\b[.,!?]?").unwrap();
    static ref COMMAND_TAB_KEY: Regex = Regex::new(r"(?i)\b(press\s+tab|hit\s+tab|tab(\s+key)?)\b[.,!?]?").unwrap();
    static ref COMMAND_ESCAPE: Regex = Regex::new(r"(?i)\b(press\s+escape|hit\s+escape|escape(\s+key)?)\b[.,!?]?").unwrap();
    static ref COMMAND_PAGE_UP: Regex = Regex::new(r"(?i)\b(page\s+up|scroll\s+up\s+(a\s+)?page)\b[.,!?]?").unwrap();
    static ref COMMAND_PAGE_DOWN: Regex = Regex::new(r"(?i)\b(page\s+down|scroll\s+down\s+(a\s+)?page)\b[.,!?]?").unwrap();

    // Cursor movement commands
    static ref COMMAND_GO_LEFT: Regex = Regex::new(r"(?i)\b(go\s+left|move\s+left|cursor\s+left|left\s+arrow)\b[.,!?]?").unwrap();
    static ref COMMAND_GO_RIGHT: Regex = Regex::new(r"(?i)\b(go\s+right|move\s+right|cursor\s+right|right\s+arrow)\b[.,!?]?").unwrap();
    static ref COMMAND_GO_UP: Regex = Regex::new(r"(?i)\b(go\s+up|move\s+up|cursor\s+up|up\s+arrow)\b[.,!?]?").unwrap();
    static ref COMMAND_GO_DOWN: Regex = Regex::new(r"(?i)\b(go\s+down|move\s+down|cursor\s+down|down\s+arrow)\b[.,!?]?").unwrap();
    static ref COMMAND_GO_START: Regex = Regex::new(r"(?i)\b(go\s+to\s+start|go\s+to\s+beginning|move\s+to\s+start|move\s+to\s+beginning|start\s+of\s+line|beginning\s+of\s+line|home(\s+key)?)\b[.,!?]?").unwrap();
    static ref COMMAND_GO_END: Regex = Regex::new(r"(?i)\b(go\s+to\s+end|move\s+to\s+end|end\s+of\s+line|end(\s+key)?)\b[.,!?]?").unwrap();
    static ref COMMAND_GO_WORD_LEFT: Regex = Regex::new(r"(?i)\b(word\s+left|move\s+word\s+left|previous\s+word|back\s+word)\b[.,!?]?").unwrap();
    static ref COMMAND_GO_WORD_RIGHT: Regex = Regex::new(r"(?i)\b(word\s+right|move\s+word\s+right|next\s+word|forward\s+word)\b[.,!?]?").unwrap();
    static ref COMMAND_SELECT_LEFT: Regex = Regex::new(r"(?i)\b(select\s+left|extend\s+left|select\s+one\s+left)\b[.,!?]?").unwrap();
    static ref COMMAND_SELECT_RIGHT: Regex = Regex::new(r"(?i)\b(select\s+right|extend\s+right|select\s+one\s+right)\b[.,!?]?").unwrap();
    static ref COMMAND_SELECT_UP: Regex = Regex::new(r"(?i)\b(select\s+up|extend\s+up)\b[.,!?]?").unwrap();
    static ref COMMAND_SELECT_DOWN: Regex = Regex::new(r"(?i)\b(select\s+down|extend\s+down)\b[.,!?]?").unwrap();
    static ref COMMAND_SELECT_WORD_LEFT: Regex = Regex::new(r"(?i)\b(select\s+(word\s+left|previous\s+word)|extend\s+(word\s+left|previous\s+word))\b[.,!?]?").unwrap();
    static ref COMMAND_SELECT_WORD_RIGHT: Regex = Regex::new(r"(?i)\b(select\s+(word\s+right|next\s+word)|extend\s+(word\s+right|next\s+word))\b[.,!?]?").unwrap();
    static ref COMMAND_SELECT_TO_START: Regex = Regex::new(r"(?i)\b(select\s+to\s+(start|beginning)|select\s+to\s+start\s+of\s+line|select\s+to\s+beginning\s+of\s+line)\b[.,!?]?").unwrap();
    static ref COMMAND_SELECT_TO_END: Regex = Regex::new(r"(?i)\b(select\s+to\s+end|select\s+to\s+end\s+of\s+line)\b[.,!?]?").unwrap();

    // Navigation/formatting commands
    static ref COMMAND_ALL_CAPS: Regex = Regex::new(r"(?i)\ball\s*caps\s+(.+?)(?:\s+end\s*caps|\s*$)").unwrap();
    static ref COMMAND_NO_CAPS: Regex = Regex::new(r"(?i)\bno\s*caps\s+(.+?)(?:\s+end\s*caps|\s*$)").unwrap();
    static ref COMMAND_CAP: Regex = Regex::new(r"(?i)\bcap\s+(\w+)").unwrap();

    // Spacing commands
    static ref COMMAND_NO_SPACE: Regex = Regex::new(r"(?i)\bno\s*space\b").unwrap();
    static ref COMMAND_SPACE: Regex = Regex::new(r"(?i)\binsert\s+space\b").unwrap();

    // Pattern: "camel case X Y Z" -> xYZ
    static ref CAMEL_CASE_PATTERN: Regex = Regex::new(
        r"(?i)\bcamel\s*case\s+([a-z]+(?:\s+[a-z]+)*)\b"
    ).unwrap();

    // Pattern: "snake case X Y Z" -> x_y_z
    static ref SNAKE_CASE_PATTERN: Regex = Regex::new(
        r"(?i)\bsnake\s*case\s+([a-z]+(?:\s+[a-z]+)*)\b"
    ).unwrap();

    // Pattern: "pascal case X Y Z" -> XYZ
    static ref PASCAL_CASE_PATTERN: Regex = Regex::new(
        r"(?i)\bpascal\s*case\s+([a-z]+(?:\s+[a-z]+)*)\b"
    ).unwrap();

    // Pattern: "kebab case X Y Z" -> x-y-z
    static ref KEBAB_CASE_PATTERN: Regex = Regex::new(
        r"(?i)\bkebab\s*case\s+([a-z]+(?:\s+[a-z]+)*)\b"
    ).unwrap();

    // Pattern: "constant case X Y Z" -> X_Y_Z
    static ref CONSTANT_CASE_PATTERN: Regex = Regex::new(
        r"(?i)\b(constant|screaming)\s*case\s+([a-z]+(?:\s+[a-z]+)*)\b"
    ).unwrap();

    // Pattern: "string X" -> "X"
    static ref STRING_PATTERN: Regex = Regex::new(
        r#"(?i)\bstring\s+"?([^"]+)"?\b"#
    ).unwrap();

    // Pattern: "variable X" or "var X" -> variable name
    static ref VARIABLE_PATTERN: Regex = Regex::new(
        r"(?i)\b(variable|var|const|let)\s+([a-z]+(?:\s+[a-z]+)*)\b"
    ).unwrap();

    // Pattern: "class X" -> ClassName
    static ref CLASS_PATTERN: Regex = Regex::new(
        r"(?i)\bclass\s+([a-z]+(?:\s+[a-z]+)*)\b"
    ).unwrap();

    // Common abbreviations (processed carefully to avoid file extensions)
    static ref ABBREV_PATTERN: Regex = Regex::new(
        r"(?i)\b(http|https|api|url|html|css|json|xml|sql|gui|cli|sdk|ide|dom|ajax|rest|crud|orm|mvc|jwt|oauth|ssr|csr|pwa|spa|seo|cdn|dns|ssh|ssl|tls|ftp|tcp|udp|ip|os|cpu|gpu|ram|ssd|hdd|usb|pdf|csv|svg|png|jpg|gif|mp3|mp4|avi|exe|dll|npm|yarn|pnpm|git|svn|aws|gcp|env)\b"
    ).unwrap();

    // Pattern: "in file X dot ext" or "file X dot ext" -> @X.ext (IDE file mention)
    // Matches patterns like: "in index dot ts", "file main dot rs", "the app dot tsx"
    static ref FILE_MENTION_PATTERN: Regex = Regex::new(
        r"(?i)\b(in|the|file|from|to|open|edit|fix|update|check|see|look at|modify|change|review|refactor|at|mention)\s+([a-z][a-z0-9_-]*)\s+dot\s+(js|ts|tsx|jsx|rs|py|go|rb|java|cpp|c|h|hpp|css|scss|html|json|yaml|yml|toml|md|txt|sh|sql|vue|svelte|astro|env|config|lock|gitignore|dockerignore|makefile)\b"
    ).unwrap();

    // Pattern: standalone file mention "X dot ext" -> @X.ext (for AI IDE context)
    // This is more aggressive - any "word dot extension" pattern
    static ref STANDALONE_FILE_MENTION_PATTERN: Regex = Regex::new(
        r"(?i)\b([a-z][a-z0-9_-]*)\s+dot\s+(js|ts|tsx|jsx|rs|py|go|rb|java|cpp|c|h|hpp|css|scss|html|json|yaml|yml|toml|md|txt|sh|sql|vue|svelte|astro|env|config|lock|gitignore|makefile)\b"
    ).unwrap();

    // Pattern for path with file mention: "src slash components slash button dot tsx" -> src/components/@button.tsx
    static ref PATH_FILE_MENTION_PATTERN: Regex = Regex::new(
        r"(?i)\b(in|the|file|from|to|open|edit|fix|update|check|see|look at|modify|change|review|refactor)\s+([a-z][a-z0-9_/-]*)\s+([a-z][a-z0-9_-]*)\s+dot\s+(js|ts|tsx|jsx|rs|py|go|rb|java|cpp|c|h|hpp|css|scss|html|json|yaml|yml|toml|md|txt|sh|sql|vue|svelte|astro)\b"
    ).unwrap();
}

impl PostProcessor {
    pub fn new() -> Self {
        let mut keywords = HashMap::new();

        // Common programming keywords
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

    /// Create a post-processor preloaded with the given custom vocabulary
    /// entries. Entries with empty `spoken` or `written` fields are
    /// silently ignored.
    pub fn with_vocabulary(entries: &[VocabularyEntry]) -> Self {
        let mut pp = Self::new();
        pp.set_vocabulary(entries);
        pp
    }

    /// Replace the active vocabulary with the provided entries. Invalid
    /// entries (empty or unparseable) are silently skipped so a single
    /// malformed term cannot disable the whole pipeline.
    pub fn set_vocabulary(&mut self, entries: &[VocabularyEntry]) {
        let mut compiled: Vec<(regex::Regex, String)> = entries
            .iter()
            .filter(|e| !e.spoken.trim().is_empty() && !e.written.is_empty())
            .filter_map(|e| {
                let escaped = regex::escape(e.spoken.trim());
                // Word boundaries: keep alphanumeric/underscore characters
                // on either side. This stops "go" inside "google" from
                // triggering a replacement while still matching
                // "go to start".
                let pattern = format!(r"(?i)\b{}\b", escaped);
                Regex::new(&pattern).ok().map(|re| (re, e.written.clone()))
            })
            .collect();

        // Longest spoken form first so multi-word terms win over their
        // single-word substrings (e.g. "next js" before "js").
        compiled.sort_by(|a, b| b.0.as_str().len().cmp(&a.0.as_str().len()));

        self.vocabulary = compiled;
    }

    /// Apply the active vocabulary to a string, replacing every spoken
    /// term with its canonical written form using case-insensitive
    /// word-boundary matching. Applied first so that all downstream
    /// transformations operate on the user's canonical text.
    fn apply_vocabulary(&self, text: &str) -> String {
        if self.vocabulary.is_empty() {
            return text.to_string();
        }
        let mut result = text.to_string();
        for (pattern, replacement) in &self.vocabulary {
            result = pattern.replace_all(&result, replacement.as_str()).to_string();
        }
        result
    }

/// Main post-processing function
    pub fn process(&self, text: &str) -> String {
        let mut result = text.to_string();

        // Apply transformations in order
        // IMPORTANT: Process file paths and mentions BEFORE sentence casing
        // to avoid capitalizing letters after dots in filenames

        // First, process voice commands (these take highest priority)
        result = self.process_voice_commands(&result);

        // Apply user-defined custom vocabulary next so the rest of the
        // pipeline operates on the user's canonical domain terms.
        result = self.apply_vocabulary(&result);

        // Then apply code-specific transformations that involve "dot"
        result = self.process_explicit_casing(&result);
        result = self.process_functions(&result);
        result = self.process_file_mentions(&result); // Process @file mentions first
        result = self.process_file_paths(&result); // Then regular file paths

        result = self.process_variables(&result);
        result = self.process_classes(&result);
        result = self.process_symbols(&result); // Convert remaining "dot" to "."

        // NOW apply sentence casing after file paths are processed
        // This prevents capitalizing after dots in filenames like "build.rs"
        result = self.fix_sentence_casing(&result);

        result = self.process_abbreviations(&result);
        result = self.process_keywords(&result); // Apply keyword casing
        result = self.cleanup_whitespace(&result);

        result
    }

/// Process only voice-command markers without applying the rest of the
    /// smart text formatting pipeline.
    pub fn extract_voice_commands(&self, text: &str) -> String {
        let mut result = self.process_voice_commands(text);
        // Still honour the user's custom vocabulary so they get consistent
        // domain terms even when voice-command-only mode is active.
        result = self.apply_vocabulary(&result);
        result
    }

    /// Process voice commands like punctuation, new line, delete, etc.
    fn process_voice_commands(&self, text: &str) -> String {
        let mut result = text.to_string();

        // Text formatting commands (process first)
        // ALL CAPS: "all caps hello world end caps" -> "HELLO WORLD"
        result = COMMAND_ALL_CAPS
            .replace_all(&result, |caps: &regex::Captures| caps[1].to_uppercase())
            .to_string();

        // no caps: "no caps HELLO WORLD end caps" -> "hello world"
        result = COMMAND_NO_CAPS
            .replace_all(&result, |caps: &regex::Captures| caps[1].to_lowercase())
            .to_string();

        // Cap next word: "cap hello" -> "Hello"
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

        // New paragraph (double newline) - process before new line
        result = NEW_PARAGRAPH_PATTERN
            .replace_all(&result, "\n\n")
            .to_string();

        // New line (single newline)
        result = NEWLINE_PATTERN.replace_all(&result, "\n").to_string();

        // Punctuation commands
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

        // Symbol commands
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

        // Spacing commands
        result = COMMAND_NO_SPACE.replace_all(&result, "").to_string();
        result = COMMAND_SPACE.replace_all(&result, " ").to_string();

        // Special action commands - these become control sequences
        // The frontend will interpret these and perform the action
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

        // Additional editing commands
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

        // Cursor movement commands
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

    /// Process explicit casing commands (camelCase, snake_case, etc.)
    fn process_explicit_casing(&self, text: &str) -> String {
        let mut result = text.to_string();

        // camelCase
        result = CAMEL_CASE_PATTERN
            .replace_all(&result, |caps: &regex::Captures| {
                self.to_camel_case(&caps[1])
            })
            .to_string();

        // snake_case
        result = SNAKE_CASE_PATTERN
            .replace_all(&result, |caps: &regex::Captures| {
                self.to_snake_case(&caps[1])
            })
            .to_string();

        // PascalCase
        result = PASCAL_CASE_PATTERN
            .replace_all(&result, |caps: &regex::Captures| {
                self.to_pascal_case(&caps[1])
            })
            .to_string();

        // kebab-case
        result = KEBAB_CASE_PATTERN
            .replace_all(&result, |caps: &regex::Captures| {
                self.to_kebab_case(&caps[1])
            })
            .to_string();

        // CONSTANT_CASE
        result = CONSTANT_CASE_PATTERN
            .replace_all(&result, |caps: &regex::Captures| {
                self.to_constant_case(&caps[2])
            })
            .to_string();

        result
    }

    /// Process function declarations
    fn process_functions(&self, text: &str) -> String {
        FUNCTION_PATTERN
            .replace_all(text, |caps: &regex::Captures| {
                let name = self.to_camel_case(&caps[2]);
                format!("{}()", name)
            })
            .to_string()
    }

    /// Process file mentions for IDE-style @ mentions (e.g., "fix bug in index dot ts" -> "fix bug in @index.ts")
    fn process_file_mentions(&self, text: &str) -> String {
        let mut result = text.to_string();

        // Process "in/file X dot ext" -> "in @X.ext"
        result = FILE_MENTION_PATTERN
            .replace_all(&result, |caps: &regex::Captures| {
                let preposition = &caps[1];
                let filename = caps[2].to_lowercase();
                let ext = caps[3].to_lowercase();
                format!("{} @{}.{}", preposition, filename, ext)
            })
            .to_string();

        // Also process standalone file mentions (without preposition)
        // This converts "build dot rs" -> "@build.rs" when no preposition is present
        // Only do this if the pattern wasn't already matched above (check for @)
        result = STANDALONE_FILE_MENTION_PATTERN
            .replace_all(&result, |caps: &regex::Captures| {
                let filename = caps[1].to_lowercase();
                let ext = caps[2].to_lowercase();
                // Check if this was already processed (would have @ before it)
                format!("@{}.{}", filename, ext)
            })
            .to_string();

        result
    }

    /// Process file paths (e.g., "index dot ts" -> "@index.ts")
    /// Always adds @ prefix for IDE file mentions
    fn process_file_paths(&self, text: &str) -> String {
        let mut result = FILE_PATH_PATTERN
            .replace_all(text, |caps: &regex::Captures| {
                format!("@{}.{}", caps[1].to_lowercase(), caps[2].to_lowercase())
            })
            .to_string();

        // Also handle files that already have a period (Whisper sometimes outputs "build.rs" directly)
        // Add @ prefix if not already present
        result = FILE_WITH_PERIOD_PATTERN
            .replace_all(&result, |caps: &regex::Captures| {
                let filename = &caps[1];
                let ext = caps[2].to_lowercase();

                // Only add @ if it's a known file extension
                if FILE_EXTENSIONS.contains(&ext.as_str()) {
                    format!("@{}.{}", filename.to_lowercase(), ext)
                } else {
                    // Not a known extension, keep as-is
                    format!("{}.{}", filename, &caps[2])
                }
            })
            .to_string();

        // Remove duplicate @@ if any
        result = result.replace("@@", "@");

        result
    }

    /// Process variable declarations
    fn process_variables(&self, text: &str) -> String {
        VARIABLE_PATTERN
            .replace_all(text, |caps: &regex::Captures| {
                let keyword = caps[1].to_lowercase();
                let name = self.to_camel_case(&caps[2]);
                format!("{} {}", keyword, name)
            })
            .to_string()
    }

    /// Process class declarations
    fn process_classes(&self, text: &str) -> String {
        CLASS_PATTERN
            .replace_all(text, |caps: &regex::Captures| {
                let name = self.to_pascal_case(&caps[1]);
                format!("class {}", name)
            })
            .to_string()
    }

    /// Process programming symbols
    fn process_symbols(&self, text: &str) -> String {
        let mut result = text.to_string();

        // Order matters - process more specific patterns first
        result = SEMICOLON_PATTERN.replace_all(&result, ";").to_string();
        result = BACKSLASH_PATTERN.replace_all(&result, "\\").to_string();
        result = SLASH_PATTERN.replace_all(&result, "/").to_string();
        result = UNDERSCORE_PATTERN.replace_all(&result, "_").to_string();
        result = HYPHEN_PATTERN.replace_all(&result, "-").to_string();
        result = COLON_PATTERN.replace_all(&result, ":").to_string();
        result = ARROW_PATTERN.replace_all(&result, "=>").to_string();
        result = EQUALS_PATTERN.replace_all(&result, "=").to_string();

        // Brackets and braces
        result = OPEN_PAREN_PATTERN.replace_all(&result, "(").to_string();
        result = CLOSE_PAREN_PATTERN.replace_all(&result, ")").to_string();
        result = OPEN_BRACE_PATTERN.replace_all(&result, "{").to_string();
        result = CLOSE_BRACE_PATTERN.replace_all(&result, "}").to_string();
        result = OPEN_SQUARE_PATTERN.replace_all(&result, "[").to_string();
        result = CLOSE_SQUARE_PATTERN.replace_all(&result, "]").to_string();

        // Tab character (newline is handled in process_voice_commands)
        result = TAB_PATTERN.replace_all(&result, "\t").to_string();

        // Process "dot" last but only standalone dots, not in file paths
        // Don't convert "dot" if it's already been processed as part of a file path
        result = self.process_standalone_dots(&result);

        result
    }

    /// Process standalone dots (not part of file paths)
    fn process_standalone_dots(&self, text: &str) -> String {
        // Only convert "dot" when it's not adjacent to a file extension
        // Preserve newlines and tabs by processing line by line
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
                    // Check if next word looks like a file extension
                    let next_is_ext = words
                        .get(i + 1)
                        .map(|w| self.file_extensions.contains(&w.to_lowercase().as_str()))
                        .unwrap_or(false);

                    // If it's before an extension, keep it (will be processed by file path)
                    // Otherwise convert to "."
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

    /// Process abbreviations to uppercase (but not file extensions after @ or .)
    fn process_abbreviations(&self, text: &str) -> String {
        let mut result = String::new();
        let mut last_end = 0;

        for cap in ABBREV_PATTERN.captures_iter(text) {
            let m = cap.get(1).unwrap();
            let start = m.start();

            // Check if preceded by @ or . (file extension context)
            let is_file_ext = if start > 0 {
                let prev_char = text.chars().nth(start - 1).unwrap_or(' ');
                prev_char == '@' || prev_char == '.'
            } else {
                false
            };

            result.push_str(&text[last_end..start]);

            if is_file_ext {
                // Keep as lowercase for file extensions
                result.push_str(&m.as_str().to_lowercase());
            } else {
                // Uppercase for abbreviations
                result.push_str(&m.as_str().to_uppercase());
            }

            last_end = m.end();
        }

        result.push_str(&text[last_end..]);
        result
    }

    /// Process programming keywords to their proper casing
    fn process_keywords(&self, text: &str) -> String {
        let mut result = String::new();
        let mut last_end = 0;

        // Simple word boundary matching for keywords
        for (i, c) in text.char_indices() {
            if c.is_alphabetic()
                && (i == 0
                    || !text
                        .chars()
                        .nth(i - 1)
                        .map(|p| p.is_alphanumeric())
                        .unwrap_or(false))
            {
                // Start of a word
                let word_start = i;
                let word_end = text[i..]
                    .char_indices()
                    .find(|(_, c)| !c.is_alphanumeric())
                    .map(|(j, _)| i + j)
                    .unwrap_or(text.len());

                let word = &text[word_start..word_end];
                let lower = word.to_lowercase();

                // Check if this word is a known keyword
                if let Some(proper_case) = self.keywords.get(&lower) {
                    // Add text before this word
                    result.push_str(&text[last_end..word_start]);
                    result.push_str(proper_case);
                    last_end = word_end;
                }
            }
        }

        // Add remaining text
        result.push_str(&text[last_end..]);

        if result.is_empty() {
            text.to_string()
        } else {
            result
        }
    }

    /// Fix sentence casing (capitalize first letter after periods)
    /// But NOT after dots in file paths/extensions
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
                    // Check if this looks like a file extension (alphanumeric before and after the dot)
                    let before_is_alnum = i > 0
                        && chars
                            .get(i - 1)
                            .map(|ch| ch.is_alphanumeric())
                            .unwrap_or(false);
                    let after_is_alnum = chars
                        .get(i + 1)
                        .map(|ch| ch.is_alphanumeric())
                        .unwrap_or(false);

                    // Only capitalize if it's a sentence ending (not a file extension)
                    // File extensions have alphanumeric chars both before and after the dot
                    if c == '.' && before_is_alnum && after_is_alnum {
                        // This looks like a file extension, don't capitalize
                        capitalize_next = false;
                    } else {
                        capitalize_next = true;
                    }
                }
            }
        }

        result
    }

    /// Clean up extra whitespace
    fn cleanup_whitespace(&self, text: &str) -> String {
        // Replace multiple spaces with single space
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

    // ========== Case conversion helpers ==========

    fn to_camel_case(&self, text: &str) -> String {
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.is_empty() {
            return String::new();
        }

        let mut result = words[0].to_lowercase();
        for word in words.iter().skip(1) {
            let mut chars = word.chars();
            if let Some(first) = chars.next() {
                result.push(first.to_uppercase().next().unwrap_or(first));
                result.extend(chars.map(|c| c.to_lowercase().next().unwrap_or(c)));
            }
        }
        result
    }

    fn to_pascal_case(&self, text: &str) -> String {
        text.split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    Some(first) => {
                        let mut s = first.to_uppercase().to_string();
                        s.extend(chars.map(|c| c.to_lowercase().next().unwrap_or(c)));
                        s
                    }
                    None => String::new(),
                }
            })
            .collect()
    }

    fn to_snake_case(&self, text: &str) -> String {
        text.split_whitespace()
            .map(|w| w.to_lowercase())
            .collect::<Vec<_>>()
            .join("_")
    }

    fn to_kebab_case(&self, text: &str) -> String {
        text.split_whitespace()
            .map(|w| w.to_lowercase())
            .collect::<Vec<_>>()
            .join("-")
    }

    fn to_constant_case(&self, text: &str) -> String {
        text.split_whitespace()
            .map(|w| w.to_uppercase())
            .collect::<Vec<_>>()
            .join("_")
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
        // When at start of text, sentence casing capitalizes first letter
        assert_eq!(pp.process("camel case hello world"), "HelloWorld");
        // In context, would be lowercase
        assert_eq!(
            pp.process("use camel case get user data"),
            "Use getUserData"
        );
    }

    #[test]
    fn test_snake_case() {
        let pp = PostProcessor::new();
        // When at start, first letter capitalized
        assert_eq!(pp.process("snake case hello world"), "Hello_world");
        // In context
        assert_eq!(pp.process("use snake case hello world"), "Use hello_world");
    }

    #[test]
    fn test_file_paths() {
        let pp = PostProcessor::new();
        // File paths get converted properly
        assert_eq!(pp.process("open index dot ts"), "Open @index.ts");
        // Standalone file mention gets @ prefix, sentence casing applies to "Main"
        assert_eq!(pp.process("main dot rs"), "@Main.rs");
    }

    #[test]
    fn test_file_mentions() {
        let pp = PostProcessor::new();
        // IDE-style file mentions with @
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
        // When at start, first letter capitalized by sentence casing
        assert_eq!(pp.process("function get user"), "GetUser()");
        // In context
        assert_eq!(pp.process("call function get user"), "Call getUser()");
    }

    #[test]
    fn test_symbols() {
        let pp = PostProcessor::new();
        // Symbols replace the word but space handling may vary
        let result = pp.process("hello slash world");
        assert!(result.contains("/"));
        let result = pp.process("a equals b");
        assert!(result.contains("="));
    }

    #[test]
    fn test_keywords() {
        let pp = PostProcessor::new();
        // Keywords maintain proper casing
        // Test with keywords that won't be matched by other patterns
        let result = pp.process("this is true and false");
        println!("Result: '{}'", result);
        assert!(result.contains("true"), "Expected 'true' in '{}'", result);
        assert!(result.contains("false"), "Expected 'false' in '{}'", result);
    }

    #[test]
    fn test_voice_commands() {
        let pp = PostProcessor::new();

        // Test newline commands through full processor
        let result = pp.process("hello new line world");
        println!("Full process result: '{:?}'", result);
        // The result should contain a newline character
        assert!(
            result.contains('\n'),
            "Expected newline character in '{}'",
            result
        );

        // Test new paragraph
        let result = pp.process("hello new paragraph world");
        println!("New paragraph result: '{:?}'", result);
        assert!(
            result.contains("\n\n"),
            "Expected double newline in '{}'",
            result
        );

        // Test action commands produce markers
        let result = pp.process("delete that");
        println!("Delete that result: '{:?}'", result);
        assert!(
            result.contains("[[DELETE_LAST]]"),
            "Expected DELETE_LAST marker in '{}'",
            result
        );

        let result = pp.process("scratch that");
        println!("Scratch that result: '{:?}'", result);
        assert!(
            result.contains("[[DELETE_LAST]]"),
            "Expected DELETE_LAST marker in '{}'",
            result
        );

        // Test simple "undo" without "that"
        let result = pp.process("undo");
        println!("Undo result: '{:?}'", result);
        assert!(
            result.contains("[[UNDO]]"),
            "Expected UNDO marker in '{}'",
            result
        );

        let result = pp.process("undo that");
        println!("Undo that result: '{:?}'", result);
        assert!(
            result.contains("[[UNDO]]"),
            "Expected UNDO marker in '{}'",
            result
        );

        // Test simple "redo"
        let result = pp.process("redo");
        println!("Redo result: '{:?}'", result);
        assert!(
            result.contains("[[REDO]]"),
            "Expected REDO marker in '{}'",
            result
        );

        // Test simple "paste"
        let result = pp.process("paste");
        println!("Paste result: '{:?}'", result);
        assert!(
            result.contains("[[PASTE]]"),
            "Expected PASTE marker in '{}'",
            result
        );

        let result = pp.process("copy");
        println!("Copy result: '{:?}'", result);
        assert!(
            result.contains("[[COPY]]"),
            "Expected COPY marker in '{}'",
            result
        );

        let result = pp.process("cut");
        println!("Cut result: '{:?}'", result);
        assert!(
            result.contains("[[CUT]]"),
            "Expected CUT marker in '{}'",
            result
        );

        let result = pp.process("enter");
        println!("Enter result: '{:?}'", result);
        assert!(
            result.contains("[[ENTER]]"),
            "Expected ENTER marker in '{}'",
            result
        );

        let result = pp.process("tab");
        println!("Tab result: '{:?}'", result);
        assert!(
            result.contains("[[TAB]]"),
            "Expected TAB marker in '{}'",
            result
        );

        let result = pp.process("escape");
        println!("Escape result: '{:?}'", result);
        assert!(
            result.contains("[[ESCAPE]]"),
            "Expected ESCAPE marker in '{}'",
            result
        );

        let result = pp.process("forward delete");
        println!("Forward delete result: '{:?}'", result);
        assert!(
            result.contains("[[DELETE_FORWARD]]"),
            "Expected DELETE_FORWARD marker in '{}'",
            result
        );

        let result = pp.process("page down");
        println!("Page down result: '{:?}'", result);
        assert!(
            result.contains("[[PAGE_DOWN]]"),
            "Expected PAGE_DOWN marker in '{}'",
            result
        );

        // Test navigation commands
        let result = pp.process("go left");
        println!("Go left result: '{:?}'", result);
        assert!(
            result.contains("[[LEFT]]"),
            "Expected LEFT marker in '{}'",
            result
        );

        let result = pp.process("go to end");
        println!("Go to end result: '{:?}'", result);
        assert!(
            result.contains("[[END]]"),
            "Expected END marker in '{}'",
            result
        );

        let result = pp.process("move to start");
        println!("Move to start result: '{:?}'", result);
        assert!(
            result.contains("[[HOME]]"),
            "Expected HOME marker in '{}'",
            result
        );

        let result = pp.process("move word right");
        println!("Move word right result: '{:?}'", result);
        assert!(
            result.contains("[[WORD_RIGHT]]"),
            "Expected WORD_RIGHT marker in '{}'",
            result
        );

        let result = pp.process("select left");
        println!("Select left result: '{:?}'", result);
        assert!(
            result.contains("[[SELECT_LEFT]]"),
            "Expected SELECT_LEFT marker in '{}'",
            result
        );

        let result = pp.process("select next word");
        println!("Select next word result: '{:?}'", result);
        assert!(
            result.contains("[[SELECT_WORD_RIGHT]]"),
            "Expected SELECT_WORD_RIGHT marker in '{}'",
            result
        );

        let result = pp.process("select to end");
        println!("Select to end result: '{:?}'", result);
        assert!(
            result.contains("[[SELECT_TO_END]]"),
            "Expected SELECT_TO_END marker in '{}'",
            result
        );
    }

    #[test]
    fn test_voice_commands_with_whisper_punctuation() {
        let pp = PostProcessor::new();

        // Whisper often adds punctuation at the end
        // Test that commands still work when period follows

        // When "new line" is the only content, it becomes empty after trim
        // (the newline is created but trimmed as trailing whitespace)
        let result = pp.process("new line.");
        println!("new line. => '{:?}'", result);
        // This is expected - standalone newline gets trimmed
        assert!(
            result.is_empty() || result == "\n",
            "Got unexpected result: '{}'",
            result
        );

        // In context, the newline is preserved
        let result = pp.process("hello new line. world");
        println!("hello new line. world => '{:?}'", result);
        assert!(
            result.contains('\n'),
            "Expected newline in context, got '{}'",
            result
        );

        let result = pp.process("delete that.");
        println!("delete that. => '{:?}'", result);
        assert_eq!(
            result.trim(),
            "[[DELETE_LAST]]",
            "Expected just marker, got '{}'",
            result
        );

        let result = pp.process("undo that.");
        println!("undo that. => '{:?}'", result);
        assert_eq!(
            result.trim(),
            "[[UNDO]]",
            "Expected just marker, got '{}'",
            result
        );
    }
}
