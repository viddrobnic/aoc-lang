use crate::completion::{CompletionItem, CompletionItemKind, InsertTextFormat};

pub fn extend_completions(completions: &mut Vec<CompletionItem>) {
    completions.push(CompletionItem {
        label: "for".to_string(),
        kind: Some(CompletionItemKind::Snippet as i32),
        insert_text: Some("for ($1; $2; $3) {\n    $4\n}$0".to_string()),
        insert_text_format: Some(InsertTextFormat::Snippet as i32),
    });

    completions.push(CompletionItem {
        label: "if".to_string(),
        kind: Some(CompletionItemKind::Snippet as i32),
        insert_text: Some("if ($1) {\n    $2\n}$0".to_string()),
        insert_text_format: Some(InsertTextFormat::Snippet as i32),
    });

    completions.push(CompletionItem {
        label: "ifelse".to_string(),
        kind: Some(CompletionItemKind::Snippet as i32),
        insert_text: Some("if ($1) {\n    $2\n} else {\n    $3\n}$0".to_string()),
        insert_text_format: Some(InsertTextFormat::Snippet as i32),
    });

    completions.push(CompletionItem {
        label: "while".to_string(),
        kind: Some(CompletionItemKind::Snippet as i32),
        insert_text: Some("while ($1) {\n    $2\n}$0".to_string()),
        insert_text_format: Some(InsertTextFormat::Snippet as i32),
    });
}
