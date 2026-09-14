use vox_ai_lib::database::Database;

fn test_database() -> (tempfile::TempDir, Database) {
    let dir = tempfile::tempdir().unwrap();
    let db = Database::new(dir.path().to_path_buf()).unwrap();
    (dir, db)
}

#[test]
fn transcription_history_add_count_and_fetch() {
    let (_dir, db) = test_database();

    let first_id = db
        .add_transcription("hello world", "base", "en", 1200)
        .unwrap();
    let second_id = db
        .add_transcription("rust backend test", "small", "en", 900)
        .unwrap();

    assert!(second_id > first_id);
    assert_eq!(db.get_transcription_history_count(None).unwrap(), 2);

    let history = db.get_transcription_history(10, 0, None).unwrap();
    assert_eq!(history.len(), 2);
    assert!(history.iter().any(|item| item.text == "hello world"));
    assert!(history.iter().any(|item| item.text == "rust backend test"));
}

#[test]
fn transcription_history_search_escapes_like_wildcards() {
    let (_dir, db) = test_database();

    db.add_transcription("literal 100% match", "base", "en", 100)
        .unwrap();
    db.add_transcription("literal 100 percent match", "base", "en", 100)
        .unwrap();
    db.add_transcription("snake_case token", "base", "en", 100)
        .unwrap();
    db.add_transcription("snakeXcase token", "base", "en", 100)
        .unwrap();

    let percent_matches = db.get_transcription_history(10, 0, Some("100%")).unwrap();
    assert_eq!(percent_matches.len(), 1);
    assert_eq!(percent_matches[0].text, "literal 100% match");

    let underscore_matches = db
        .get_transcription_history(10, 0, Some("snake_case"))
        .unwrap();
    assert_eq!(underscore_matches.len(), 1);
    assert_eq!(underscore_matches[0].text, "snake_case token");
}

#[test]
fn transcription_history_delete_and_clear() {
    let (_dir, db) = test_database();

    let first_id = db
        .add_transcription("keep for now", "base", "en", 1200)
        .unwrap();
    db.add_transcription("delete later", "base", "en", 1200)
        .unwrap();

    db.delete_transcription(first_id).unwrap();
    assert_eq!(db.get_transcription_history_count(None).unwrap(), 1);
    assert!(!db
        .get_transcription_history(10, 0, None)
        .unwrap()
        .iter()
        .any(|item| item.id == first_id));

    db.clear_transcription_history().unwrap();
    assert_eq!(db.get_transcription_history_count(None).unwrap(), 0);
}

#[test]
fn transcription_history_paginates() {
    let (_dir, db) = test_database();

    for index in 0..5 {
        db.add_transcription(&format!("item {}", index), "base", "en", 100)
            .unwrap();
    }

    let first_page = db.get_transcription_history(2, 0, None).unwrap();
    let second_page = db.get_transcription_history(2, 2, None).unwrap();

    assert_eq!(first_page.len(), 2);
    assert_eq!(second_page.len(), 2);
    assert_ne!(first_page[0].id, second_page[0].id);
}
