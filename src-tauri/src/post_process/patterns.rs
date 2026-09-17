use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    pub static ref FUNCTION_PATTERN: Regex = Regex::new(
        r"(?i)\b(function|func|method|def)\s+([a-z]+(?:\s+[a-z]+)*)\b"
    ).unwrap();

    pub static ref FILE_PATH_PATTERN: Regex = Regex::new(
        r"(?i)\b([a-z][a-z0-9_-]*)\s+dot\s+(js|ts|tsx|jsx|rs|py|go|rb|java|cpp|c|h|hpp|css|scss|html|json|yaml|yml|toml|md|txt|sh|bash|sql|vue|svelte|astro)\b"
    ).unwrap();

    pub static ref FILE_WITH_PERIOD_PATTERN: Regex = Regex::new(
        r"(?i)\b([a-z][a-z0-9_-]*)\.([a-z]{1,5})\b"
    ).unwrap();

    pub static ref FILE_EXTENSIONS: Vec<&'static str> = vec![
        "js", "ts", "tsx", "jsx", "rs", "py", "go", "rb", "java", "cpp", "c",
        "h", "hpp", "css", "scss", "html", "json", "yaml", "yml", "toml", "md",
        "txt", "sh", "bash", "sql", "vue", "svelte", "astro", "env", "lock"
    ];

    pub static ref SLASH_PATTERN: Regex = Regex::new(r"(?i)\b(forward\s+)?slash\b").unwrap();

    pub static ref BACKSLASH_PATTERN: Regex = Regex::new(r"(?i)\bback\s*slash\b").unwrap();

    pub static ref UNDERSCORE_PATTERN: Regex = Regex::new(r"(?i)\bunderscore\b").unwrap();

    pub static ref HYPHEN_PATTERN: Regex = Regex::new(r"(?i)\b(hyphen|dash)\b").unwrap();

    pub static ref DOT_PATTERN: Regex = Regex::new(r"(?i)\b(dot|period)\b").unwrap();

    pub static ref COLON_PATTERN: Regex = Regex::new(r"(?i)\bcolon\b").unwrap();

    pub static ref SEMICOLON_PATTERN: Regex = Regex::new(r"(?i)\bsemi\s*colon\b").unwrap();

    pub static ref EQUALS_PATTERN: Regex = Regex::new(r"(?i)\b(equals?(\s+sign)?|equal\s+to)\b").unwrap();

    pub static ref ARROW_PATTERN: Regex = Regex::new(r"(?i)\b(fat\s+)?arrow\b").unwrap();

    pub static ref OPEN_PAREN_PATTERN: Regex = Regex::new(r"(?i)\bopen\s*(paren|parenthesis|bracket)\b").unwrap();

    pub static ref CLOSE_PAREN_PATTERN: Regex = Regex::new(r"(?i)\bclose\s*(paren|parenthesis|bracket)\b").unwrap();

    pub static ref OPEN_BRACE_PATTERN: Regex = Regex::new(r"(?i)\bopen\s*(brace|curly)\b").unwrap();

    pub static ref CLOSE_BRACE_PATTERN: Regex = Regex::new(r"(?i)\bclose\s*(brace|curly)\b").unwrap();

    pub static ref OPEN_SQUARE_PATTERN: Regex = Regex::new(r"(?i)\bopen\s*square(\s*bracket)?\b").unwrap();

    pub static ref CLOSE_SQUARE_PATTERN: Regex = Regex::new(r"(?i)\bclose\s*square(\s*bracket)?\b").unwrap();

    pub static ref NEWLINE_PATTERN: Regex = Regex::new(r"(?i)\bnew\s*line\b[.,!?]?").unwrap();

    pub static ref NEW_PARAGRAPH_PATTERN: Regex = Regex::new(r"(?i)\bnew\s*paragraph\b[.,!?]?").unwrap();

    pub static ref TAB_PATTERN: Regex = Regex::new(r"(?i)\btab\s+(character|key)\b").unwrap();

    pub static ref COMMAND_COMMA: Regex = Regex::new(r"(?i)\binsert\s+comma\b").unwrap();
    pub static ref COMMAND_PERIOD: Regex = Regex::new(r"(?i)\binsert\s+(period|full\s+stop)\b").unwrap();
    pub static ref COMMAND_QUESTION_MARK: Regex = Regex::new(r"(?i)\binsert\s+question\s*mark\b").unwrap();
    pub static ref COMMAND_EXCLAMATION: Regex = Regex::new(r"(?i)\binsert\s+(exclamation\s*(mark|point)?|bang)\b").unwrap();
    pub static ref COMMAND_APOSTROPHE: Regex = Regex::new(r"(?i)\binsert\s+apostrophe\b").unwrap();
    pub static ref COMMAND_QUOTE: Regex = Regex::new(r"(?i)\binsert\s+(double\s+)?quote\b").unwrap();
    pub static ref COMMAND_SINGLE_QUOTE: Regex = Regex::new(r"(?i)\binsert\s+single\s+quote\b").unwrap();
    pub static ref COMMAND_OPEN_QUOTE: Regex = Regex::new(r"(?i)\bopen\s+(double\s+)?quote\b").unwrap();
    pub static ref COMMAND_CLOSE_QUOTE: Regex = Regex::new(r"(?i)\bclose\s+(double\s+)?quote\b").unwrap();
    pub static ref COMMAND_ELLIPSIS: Regex = Regex::new(r"(?i)\binsert\s+ellipsis\b").unwrap();
    pub static ref COMMAND_AMPERSAND: Regex = Regex::new(r"(?i)\binsert\s+ampersand\b").unwrap();
    pub static ref COMMAND_AT_SIGN: Regex = Regex::new(r"(?i)\binsert\s+at\s*sign\b").unwrap();
    pub static ref COMMAND_HASH: Regex = Regex::new(r"(?i)\binsert\s+(hash|hashtag|pound\s*sign|number\s*sign)\b").unwrap();
    pub static ref COMMAND_PERCENT: Regex = Regex::new(r"(?i)\binsert\s+percent(\s*sign)?\b").unwrap();
    pub static ref COMMAND_DOLLAR: Regex = Regex::new(r"(?i)\binsert\s+dollar(\s*sign)?\b").unwrap();
    pub static ref COMMAND_ASTERISK: Regex = Regex::new(r"(?i)\binsert\s+(asterisk|star)\b").unwrap();
    pub static ref COMMAND_PLUS: Regex = Regex::new(r"(?i)\binsert\s+plus(\s*sign)?\b").unwrap();
    pub static ref COMMAND_MINUS: Regex = Regex::new(r"(?i)\binsert\s+minus(\s*sign)?\b").unwrap();
    pub static ref COMMAND_TILDE: Regex = Regex::new(r"(?i)\binsert\s+tilde\b").unwrap();
    pub static ref COMMAND_CARET: Regex = Regex::new(r"(?i)\binsert\s+caret\b").unwrap();
    pub static ref COMMAND_PIPE: Regex = Regex::new(r"(?i)\binsert\s+(pipe|vertical\s*bar)\b").unwrap();
    pub static ref COMMAND_LESS_THAN: Regex = Regex::new(r"(?i)\binsert\s+(less\s*than|left\s*angle(\s*bracket)?)\b").unwrap();
    pub static ref COMMAND_GREATER_THAN: Regex = Regex::new(r"(?i)\binsert\s+(greater\s*than|right\s*angle(\s*bracket)?)\b").unwrap();

    pub static ref COMMAND_DELETE_THAT: Regex = Regex::new(r"(?i)\b(delete\s+that|scratch\s+that|remove\s+that|delete\s+last|scratch\s+last)\b[.,!?]?").unwrap();
    pub static ref COMMAND_UNDO: Regex = Regex::new(r"(?i)\bundo(\s+(that|last|it))?\b[.,!?]?").unwrap();
    pub static ref COMMAND_REDO: Regex = Regex::new(r"(?i)\bredo(\s+(that|last|it))?\b[.,!?]?").unwrap();
    pub static ref COMMAND_SELECT_ALL: Regex = Regex::new(r"(?i)\bselect\s+all(\s+text)?\b[.,!?]?").unwrap();
    pub static ref COMMAND_COPY_THAT: Regex = Regex::new(r"(?i)\bcopy(\s+(that|this|selection|it))?\b[.,!?]?").unwrap();
    pub static ref COMMAND_CUT_THAT: Regex = Regex::new(r"(?i)\bcut(\s+(that|this|selection|it))?\b[.,!?]?").unwrap();
    pub static ref COMMAND_PASTE_THAT: Regex = Regex::new(r"(?i)\bpaste(\s+(that|here|it))?\b[.,!?]?").unwrap();

    pub static ref COMMAND_BACKSPACE: Regex = Regex::new(r"(?i)\b(backspace|delete\s+character|remove\s+character)\b[.,!?]?").unwrap();
    pub static ref COMMAND_DELETE_FORWARD: Regex = Regex::new(r"(?i)\b(forward\s+delete|delete\s+forward|delete\s+next\s+character|delete\s+key)\b[.,!?]?").unwrap();
    pub static ref COMMAND_DELETE_WORD: Regex = Regex::new(r"(?i)\b(delete\s+word|remove\s+word|backspace\s+word)\b[.,!?]?").unwrap();
    pub static ref COMMAND_DELETE_LINE: Regex = Regex::new(r"(?i)\b(delete\s+line|remove\s+line|clear\s+line)\b[.,!?]?").unwrap();
    pub static ref COMMAND_ENTER: Regex = Regex::new(r"(?i)\b(press\s+enter|hit\s+enter|enter(\s+key)?)\b[.,!?]?").unwrap();
    pub static ref COMMAND_TAB_KEY: Regex = Regex::new(r"(?i)\b(press\s+tab|hit\s+tab|tab(\s+key)?)\b[.,!?]?").unwrap();
    pub static ref COMMAND_ESCAPE: Regex = Regex::new(r"(?i)\b(press\s+escape|hit\s+escape|escape(\s+key)?)\b[.,!?]?").unwrap();
    pub static ref COMMAND_PAGE_UP: Regex = Regex::new(r"(?i)\b(page\s+up|scroll\s+up\s+(a\s+)?page)\b[.,!?]?").unwrap();
    pub static ref COMMAND_PAGE_DOWN: Regex = Regex::new(r"(?i)\b(page\s+down|scroll\s+down\s+(a\s+)?page)\b[.,!?]?").unwrap();

    pub static ref COMMAND_GO_LEFT: Regex = Regex::new(r"(?i)\b(go\s+left|move\s+left|cursor\s+left|left\s+arrow)\b[.,!?]?").unwrap();
    pub static ref COMMAND_GO_RIGHT: Regex = Regex::new(r"(?i)\b(go\s+right|move\s+right|cursor\s+right|right\s+arrow)\b[.,!?]?").unwrap();
    pub static ref COMMAND_GO_UP: Regex = Regex::new(r"(?i)\b(go\s+up|move\s+up|cursor\s+up|up\s+arrow)\b[.,!?]?").unwrap();
    pub static ref COMMAND_GO_DOWN: Regex = Regex::new(r"(?i)\b(go\s+down|move\s+down|cursor\s+down|down\s+arrow)\b[.,!?]?").unwrap();
    pub static ref COMMAND_GO_START: Regex = Regex::new(r"(?i)\b(go\s+to\s+start|go\s+to\s+beginning|move\s+to\s+start|move\s+to\s+beginning|start\s+of\s+line|beginning\s+of\s+line|home(\s+key)?)\b[.,!?]?").unwrap();
    pub static ref COMMAND_GO_END: Regex = Regex::new(r"(?i)\b(go\s+to\s+end|move\s+to\s+end|end\s+of\s+line|end(\s+key)?)\b[.,!?]?").unwrap();
    pub static ref COMMAND_GO_WORD_LEFT: Regex = Regex::new(r"(?i)\b(word\s+left|move\s+word\s+left|previous\s+word|back\s+word)\b[.,!?]?").unwrap();
    pub static ref COMMAND_GO_WORD_RIGHT: Regex = Regex::new(r"(?i)\b(word\s+right|move\s+word\s+right|next\s+word|forward\s+word)\b[.,!?]?").unwrap();
    pub static ref COMMAND_SELECT_LEFT: Regex = Regex::new(r"(?i)\b(select\s+left|extend\s+left|select\s+one\s+left)\b[.,!?]?").unwrap();
    pub static ref COMMAND_SELECT_RIGHT: Regex = Regex::new(r"(?i)\b(select\s+right|extend\s+right|select\s+one\s+right)\b[.,!?]?").unwrap();
    pub static ref COMMAND_SELECT_UP: Regex = Regex::new(r"(?i)\b(select\s+up|extend\s+up)\b[.,!?]?").unwrap();
    pub static ref COMMAND_SELECT_DOWN: Regex = Regex::new(r"(?i)\b(select\s+down|extend\s+down)\b[.,!?]?").unwrap();
    pub static ref COMMAND_SELECT_WORD_LEFT: Regex = Regex::new(r"(?i)\b(select\s+(word\s+left|previous\s+word)|extend\s+(word\s+left|previous\s+word))\b[.,!?]?").unwrap();
    pub static ref COMMAND_SELECT_WORD_RIGHT: Regex = Regex::new(r"(?i)\b(select\s+(word\s+right|next\s+word)|extend\s+(word\s+right|next\s+word))\b[.,!?]?").unwrap();
    pub static ref COMMAND_SELECT_TO_START: Regex = Regex::new(r"(?i)\b(select\s+to\s+(start|beginning)|select\s+to\s+start\s+of\s+line|select\s+to\s+beginning\s+of\s+line)\b[.,!?]?").unwrap();
    pub static ref COMMAND_SELECT_TO_END: Regex = Regex::new(r"(?i)\b(select\s+to\s+end|select\s+to\s+end\s+of\s+line)\b[.,!?]?").unwrap();

    pub static ref COMMAND_ALL_CAPS: Regex = Regex::new(r"(?i)\ball\s*caps\s+(.+?)(?:\s+end\s*caps|\s*$)").unwrap();
    pub static ref COMMAND_NO_CAPS: Regex = Regex::new(r"(?i)\bno\s*caps\s+(.+?)(?:\s+end\s*caps|\s*$)").unwrap();
    pub static ref COMMAND_CAP: Regex = Regex::new(r"(?i)\bcap\s+(\w+)").unwrap();

    pub static ref COMMAND_NO_SPACE: Regex = Regex::new(r"(?i)\bno\s*space\b").unwrap();
    pub static ref COMMAND_SPACE: Regex = Regex::new(r"(?i)\binsert\s+space\b").unwrap();

    pub static ref CAMEL_CASE_PATTERN: Regex = Regex::new(
        r"(?i)\bcamel\s*case\s+([a-z]+(?:\s+[a-z]+)*)\b"
    ).unwrap();

    pub static ref SNAKE_CASE_PATTERN: Regex = Regex::new(
        r"(?i)\bsnake\s*case\s+([a-z]+(?:\s+[a-z]+)*)\b"
    ).unwrap();

    pub static ref PASCAL_CASE_PATTERN: Regex = Regex::new(
        r"(?i)\bpascal\s*case\s+([a-z]+(?:\s+[a-z]+)*)\b"
    ).unwrap();

    pub static ref KEBAB_CASE_PATTERN: Regex = Regex::new(
        r"(?i)\bkebab\s*case\s+([a-z]+(?:\s+[a-z]+)*)\b"
    ).unwrap();

    pub static ref CONSTANT_CASE_PATTERN: Regex = Regex::new(
        r"(?i)\b(constant|screaming)\s*case\s+([a-z]+(?:\s+[a-z]+)*)\b"
    ).unwrap();

    pub static ref STRING_PATTERN: Regex = Regex::new(
        r#"(?i)\bstring\s+"?([^"]+)"?\b"#
    ).unwrap();

    pub static ref VARIABLE_PATTERN: Regex = Regex::new(
        r"(?i)\b(variable|var|const|let)\s+([a-z]+(?:\s+[a-z]+)*)\b"
    ).unwrap();

    pub static ref CLASS_PATTERN: Regex = Regex::new(
        r"(?i)\bclass\s+([a-z]+(?:\s+[a-z]+)*)\b"
    ).unwrap();

    pub static ref ABBREV_PATTERN: Regex = Regex::new(
        r"(?i)\b(http|https|api|url|html|css|json|xml|sql|gui|cli|sdk|ide|dom|ajax|rest|crud|orm|mvc|jwt|oauth|ssr|csr|pwa|spa|seo|cdn|dns|ssh|ssl|tls|ftp|tcp|udp|ip|os|cpu|gpu|ram|ssd|hdd|usb|pdf|csv|svg|png|jpg|gif|mp3|mp4|avi|exe|dll|npm|yarn|pnpm|git|svn|aws|gcp|env)\b"
    ).unwrap();

    pub static ref FILE_MENTION_PATTERN: Regex = Regex::new(
        r"(?i)\b(in|the|file|from|to|open|edit|fix|update|check|see|look at|modify|change|review|refactor|at|mention)\s+([a-z][a-z0-9_-]*)\s+dot\s+(js|ts|tsx|jsx|rs|py|go|rb|java|cpp|c|h|hpp|css|scss|html|json|yaml|yml|toml|md|txt|sh|sql|vue|svelte|astro|env|config|lock|gitignore|dockerignore|makefile)\b"
    ).unwrap();

    pub static ref STANDALONE_FILE_MENTION_PATTERN: Regex = Regex::new(
        r"(?i)\b([a-z][a-z0-9_-]*)\s+dot\s+(js|ts|tsx|jsx|rs|py|go|rb|java|cpp|c|h|hpp|css|scss|html|json|yaml|yml|toml|md|txt|sh|sql|vue|svelte|astro|env|config|lock|gitignore|makefile)\b"
    ).unwrap();

    pub static ref PATH_FILE_MENTION_PATTERN: Regex = Regex::new(
        r"(?i)\b(in|the|file|from|to|open|edit|fix|update|check|see|look at|modify|change|review|refactor)\s+([a-z][a-z0-9_/-]*)\s+([a-z][a-z0-9_-]*)\s+dot\s+(js|ts|tsx|jsx|rs|py|go|rb|java|cpp|c|h|hpp|css|scss|html|json|yaml|yml|toml|md|txt|sh|sql|vue|svelte|astro)\b"
    ).unwrap();
}
