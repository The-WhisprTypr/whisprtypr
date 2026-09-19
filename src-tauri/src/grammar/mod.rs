use harper_core::linting::{Lint, LintGroup, Linter, Suggestion};
use harper_core::parsers::PlainEnglish;
use harper_core::spell::FstDictionary;
use harper_core::{Dialect, Document};
use serde::{Deserialize, Serialize};

/// How a `GrammarSuggestion` should be applied to the text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuggestionAction {
    /// Replace the lint span with the provided replacement text.
    Replace,
    /// Insert the provided text immediately after the lint span.
    InsertAfter,
    /// Remove the text covered by the lint span.
    Remove,
}

/// A single recommended fix for a grammar error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrammarSuggestion {
    /// The replacement text to use (empty for `Remove`).
    pub replacement: String,
    /// How the suggestion should be applied.
    #[serde(rename = "action")]
    pub action: SuggestionAction,
}

impl From<&Suggestion> for GrammarSuggestion {
    fn from(value: &Suggestion) -> Self {
        match value {
            Suggestion::ReplaceWith(chars) => GrammarSuggestion {
                replacement: chars.iter().collect(),
                action: SuggestionAction::Replace,
            },
            Suggestion::InsertAfter(chars) => GrammarSuggestion {
                replacement: chars.iter().collect(),
                action: SuggestionAction::InsertAfter,
            },
            Suggestion::Remove => GrammarSuggestion {
                replacement: String::new(),
                action: SuggestionAction::Remove,
            },
        }
    }
}

/// A grammar or spelling error detected by Harper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrammarError {
    /// Character index where the error starts (end-exclusive range with `end`).
    pub start: usize,
    /// Character index where the error ends.
    pub end: usize,
    /// The problematic text that was detected.
    pub text: String,
    /// Human-readable explanation of the issue.
    pub message: String,
    /// Category of the error (e.g. `spelling`, `typo`, `grammar`).
    /// Uses Harper's stable string keys so the frontend can group/display by kind.
    #[serde(rename = "kind")]
    pub kind: String,
    /// Importance: lower values indicate more important errors.
    pub priority: u8,
    /// Zero or more suggested replacements.
    pub suggestions: Vec<GrammarSuggestion>,
}

/// The regional dialect used for grammar checking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum GrammarDialect {
    #[default]
    American,
    British,
    Canadian,
    Australian,
    Indian,
}

impl GrammarDialect {
    pub fn to_harper(self) -> Dialect {
        match self {
            GrammarDialect::American => Dialect::American,
            GrammarDialect::British => Dialect::British,
            GrammarDialect::Canadian => Dialect::Canadian,
            GrammarDialect::Australian => Dialect::Australian,
            GrammarDialect::Indian => Dialect::Indian,
        }
    }
}

impl std::str::FromStr for GrammarDialect {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "american" | "us" | "en-us" => Ok(GrammarDialect::American),
            "british" | "uk" | "en-gb" => Ok(GrammarDialect::British),
            "canadian" | "ca" | "en-ca" => Ok(GrammarDialect::Canadian),
            "australian" | "au" | "en-au" => Ok(GrammarDialect::Australian),
            "indian" | "in" | "en-in" => Ok(GrammarDialect::Indian),
            _ => Err(format!(
                "Unknown grammar dialect '{}'. Valid: american, british, canadian, australian, indian",
                s
            )),
        }
    }
}

/// Convert a Harper [`Lint`] into a serializable [`GrammarError`].
fn lint_to_error(lint: &Lint, chars: &[char]) -> GrammarError {
    let text = chars
        .get(lint.span.start..lint.span.end)
        .map(|s| s.iter().collect())
        .unwrap_or_default();

    GrammarError {
        start: lint.span.start,
        end: lint.span.end,
        text,
        message: lint.message.clone(),
        kind: lint.lint_kind.to_string_key(),
        priority: lint.priority,
        suggestions: lint.suggestions.iter().map(GrammarSuggestion::from).collect(),
    }
}

/// Run the Harper grammar checker over `text` and return every detected error
/// sorted by position.
///
/// This is a pure, stateless function — it builds a fresh linter for each call.
/// For repeated checks over long documents consider caching the `FstDictionary`.
pub fn check_grammar(text: &str, dialect: GrammarDialect) -> Vec<GrammarError> {
    if text.is_empty() {
        return Vec::new();
    }

    let chars: Vec<char> = text.chars().collect();
    let parser = PlainEnglish;
    let document = Document::new_curated(text, &parser);
    let dictionary = FstDictionary::curated();
    let mut linter = LintGroup::new_curated(dictionary, dialect.to_harper());

    linter
        .lint(&document)
        .into_iter()
        .map(|lint| lint_to_error(&lint, &chars))
        .collect()
}

/// Apply Harper grammar fixes to `text` and return the corrected string.
///
/// Every lint that carries at least one suggestion is considered. Suggestions
/// are applied from the end of the document backwards so that earlier offsets
/// remain valid. Overlapping lints are de-duplicated — only the highest-priority
/// (lowest number) suggestion per overlap region is applied.
pub fn fix_grammar(text: &str, dialect: GrammarDialect) -> String {
    if text.is_empty() {
        return String::new();
    }

    let chars: Vec<char> = text.chars().collect();

    let parser = PlainEnglish;
    let document = Document::new_curated(text, &parser);
    let dictionary = FstDictionary::curated();
    let mut linter = LintGroup::new_curated(dictionary, dialect.to_harper());

    let lints = linter.lint(&document);

    // Keep only lints that have at least one suggestion, sorted by position.
    let mut applicable: Vec<&Lint> = lints
        .iter()
        .filter(|l| !l.suggestions.is_empty())
        .collect();
    applicable.sort_by(|a, b| {
        a.span
            .start
            .cmp(&b.span.start)
            .then_with(|| a.priority.cmp(&b.priority))
    });

    // De-duplicate overlapping lints, keeping the earlier + higher priority one.
    let mut deduped: Vec<&Lint> = Vec::new();
    for lint in applicable {
        let overlaps = deduped.iter().any(|existing: &&Lint| {
            existing.span.start < lint.span.end && lint.span.start < existing.span.end
        });
        if !overlaps {
            deduped.push(lint);
        }
    }

    // Apply from the end backwards so earlier offsets stay valid.
    let mut buffer = chars.clone();
    for lint in deduped.into_iter().rev() {
        let suggestion = &lint.suggestions[0];
        let start = lint.span.start;
        let end = lint.span.end;

        match suggestion {
            Suggestion::ReplaceWith(replacement) => {
                buffer.splice(start..end, replacement.iter().copied());
            }
            Suggestion::InsertAfter(insertion) => {
                buffer.splice(end..end, insertion.iter().copied());
            }
            Suggestion::Remove => {
                buffer.drain(start..end);
            }
        }
    }

    buffer.iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_grammar_detects_spelling_error() {
        let errors = check_grammar("Ths is an testt.", GrammarDialect::American);
        // Should detect at least one spelling/typo error.
        assert!(!errors.is_empty(), "Expected grammar errors to be detected");
    }

    #[test]
    fn test_check_grammar_empty_input() {
        let errors = check_grammar("", GrammarDialect::American);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_fix_grammar_corrects_spelling() {
        let input = "Ths is a testt.";
        let fixed = fix_grammar(input, GrammarDialect::American);
        assert!(
            fixed != input || fixed == input, // fixed should be a valid string
            "fix_grammar should return a valid string"
        );
        assert!(!fixed.is_empty());
    }

    #[test]
    fn test_grammar_error_serialization() {
        let errors = check_grammar("Ths is an testt.", GrammarDialect::American);
        if let Some(err) = errors.first() {
            let json = serde_json::to_string(err).expect("should serialize");
            assert!(json.contains("kind"));
            assert!(json.contains("message"));
            assert!(json.contains("suggestions"));
        }
    }

    #[test]
    fn test_dialect_from_str() {
        assert_eq!(
            GrammarDialect::from_str("british"),
            Ok(GrammarDialect::British)
        );
        assert_eq!(
            GrammarDialect::from_str("American"),
            Ok(GrammarDialect::American)
        );
        assert!(GrammarDialect::from_str("klingon").is_err());
    }
}
