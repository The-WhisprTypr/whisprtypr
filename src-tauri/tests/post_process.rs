use whisprtypr_lib::post_process::{PostProcessor, VocabularyEntry};

fn processor() -> PostProcessor {
    PostProcessor::new()
}

fn with_vocab(entries: &[(&str, &str)]) -> PostProcessor {
    let vocab: Vec<VocabularyEntry> = entries
        .iter()
        .map(|(s, w)| VocabularyEntry::new(*s, *w))
        .collect();
    PostProcessor::with_vocabulary(&vocab)
}

#[test]
fn processes_programming_case_commands() {
    let pp = processor();

    assert_eq!(pp.process("camel case user profile id"), "UserProfileId");
    assert_eq!(pp.process("snake case user profile id"), "User_profile_id");
    assert_eq!(pp.process("pascal case user profile id"), "UserProfileId");
    assert_eq!(pp.process("kebab case user profile id"), "User-profile-id");
    assert_eq!(
        pp.process("constant case user profile id"),
        "USER_PROFILE_ID"
    );

    assert_eq!(
        pp.process("use camel case user profile id"),
        "Use userProfileId"
    );
    assert_eq!(
        pp.process("use snake case user profile id"),
        "Use user_profile_id"
    );
}

#[test]
fn processes_code_constructs() {
    let pp = processor();

    assert_eq!(pp.process("function get user name"), "GetUserName()");
    assert_eq!(
        pp.process("call function get user name"),
        "Call getUserName()"
    );
    assert_eq!(
        pp.process("variable current user id"),
        "Variable currentUserId"
    );
    assert_eq!(
        pp.process("class user profile service"),
        "class UserProfileService"
    );
}

#[test]
fn processes_file_mentions_and_paths_before_sentence_casing() {
    let pp = processor();

    assert_eq!(pp.process("open index dot ts"), "Open @index.ts");
    assert_eq!(pp.process("build dot rs"), "@Build.rs");
    assert_eq!(
        pp.process("fix bug in index dot ts"),
        "Fix bug in @index.ts"
    );
}

#[test]
fn processes_symbols_and_spacing_commands() {
    let pp = processor();

    assert_eq!(
        pp.process("foo underscore bar equals value semicolon"),
        "Foo _ bar = value ;"
    );
    assert_eq!(
        pp.process("insert at sign example dot com"),
        "@ Example . Com"
    );
    assert_eq!(pp.process("hello no space world"), "Hello world");
}

#[test]
fn processes_voice_action_commands() {
    let pp = processor();

    assert_eq!(pp.process("delete that"), "[[DELETE_LAST]]");
    assert_eq!(pp.process("undo that"), "[[UNDO]]");
    assert_eq!(pp.process("redo last"), "[[REDO]]");
    assert_eq!(pp.process("select all text"), "[[SELECT_ALL]]");
    assert_eq!(pp.process("copy that"), "[[COPY]]");
    assert_eq!(pp.process("cut that"), "[[CUT]]");
    assert_eq!(pp.process("paste here"), "[[PASTE]]");
}

#[test]
fn processes_newlines_and_punctuation_commands() {
    let pp = processor();

    assert_eq!(pp.process("hello insert comma world"), "Hello , world");
    assert_eq!(pp.process("hello insert question mark"), "Hello ?");
    assert_eq!(pp.process("first new line second"), "First\nsecond");
    assert_eq!(pp.process("first new paragraph second"), "First\n\nsecond");
}

#[test]
fn preserves_common_technical_abbreviations() {
    let pp = processor();

    assert_eq!(pp.process("call the api url"), "Call the API URL");
    assert_eq!(pp.process("parse json and html"), "Parse JSON and HTML");
}

#[test]
fn custom_vocabulary_replaces_known_terms() {
    let pp = with_vocab(&[
        ("next js", "Next.js"),
        ("tauri", "Tauri"),
        ("wave e", "WhisprTypr"),
    ]);

    assert_eq!(pp.process("i love next js"), "I love Next.js");
    assert_eq!(pp.process("built with tauri and react"), "Built with Tauri and react");
    assert_eq!(
        pp.process("wave e is great"),
        "WhisprTypr is great"
    );
}

#[test]
fn custom_vocabulary_is_case_insensitive_and_word_boundary_aware() {
    let pp = with_vocab(&[("oauth", "OAuth")]);

    // Exact and case-variant forms both get replaced.
    assert_eq!(pp.process("use oauth here"), "Use OAuth here");
    assert_eq!(pp.process("OAUTH is tricky"), "OAuth is tricky");

    // Substring matches inside larger tokens are NOT replaced.
    assert_eq!(
        pp.process("oauthenticator was wrong"),
        "Oauthenticator was wrong"
    );
}

#[test]
fn custom_vocabulary_prefers_longer_phrases() {
    let pp = with_vocab(&[("js", "JS"), ("next js", "Next.js")]);

    // The two-word phrase wins over the single-word entry that happens
    // to be a substring of it.
    assert_eq!(pp.process("we use next js here"), "We use Next.js here");
    // Single-word entries still match standalone occurrences.
    assert_eq!(pp.process("plain js rocks"), "Plain JS rocks");
}

#[test]
fn custom_vocabulary_skips_empty_entries() {
    let pp = with_vocab(&[
        ("", "ignored"),
        ("   ", "ignored"),
        ("real", "Real"),
    ]);

    assert_eq!(pp.process("that is real"), "That is Real");
}

#[test]
fn custom_vocabulary_preserves_canonical_written_form() {
    let pp = with_vocab(&[("k eight s", "k8s")]);

    // The user's canonical written form is preserved verbatim, even
    // when the surrounding text would otherwise influence casing.
    assert_eq!(pp.process("deploy to k eight s"), "Deploy to k8s");
}

#[test]
fn custom_vocabulary_runs_in_voice_command_extractor() {
    let pp = with_vocab(&[("wave e", "WhisprTypr")]);

    let result = pp.extract_voice_commands("wave e delete that");
    assert!(result.contains("WhisprTypr"), "got: {:?}", result);
    assert!(
        result.contains("[[DELETE_LAST]]"),
        "expected DELETE_LAST marker in {:?}",
        result
    );
}
