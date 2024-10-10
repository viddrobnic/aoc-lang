use runtime::builtin::Builtin;

use crate::{
    hover::MarkupContent,
    message::completion::{CompletionItem, CompletionItemKind, InsertTextFormat},
};

pub fn extend_snippets(completions: &mut Vec<CompletionItem>) {
    completions.push(CompletionItem {
        label: "for".to_string(),
        kind: Some(CompletionItemKind::Snippet as i32),
        insert_text: Some("for ($1; $2; $3) {\n    $4\n}$0".to_string()),
        insert_text_format: Some(InsertTextFormat::Snippet as i32),
        documentation: None,
    });

    completions.push(CompletionItem {
        label: "if".to_string(),
        kind: Some(CompletionItemKind::Snippet as i32),
        insert_text: Some("if ($1) {\n    $2\n}$0".to_string()),
        insert_text_format: Some(InsertTextFormat::Snippet as i32),
        documentation: None,
    });

    completions.push(CompletionItem {
        label: "ifelse".to_string(),
        kind: Some(CompletionItemKind::Snippet as i32),
        insert_text: Some("if ($1) {\n    $2\n} else {\n    $3\n}$0".to_string()),
        insert_text_format: Some(InsertTextFormat::Snippet as i32),
        documentation: None,
    });

    completions.push(CompletionItem {
        label: "while".to_string(),
        kind: Some(CompletionItemKind::Snippet as i32),
        insert_text: Some("while ($1) {\n    $2\n}$0".to_string()),
        insert_text_format: Some(InsertTextFormat::Snippet as i32),
        documentation: None,
    });
}

pub fn extend_builtin(completions: &mut Vec<CompletionItem>) {
    completions.push(CompletionItem {
        label: "len".to_string(),
        kind: Some(CompletionItemKind::Function as i32),
        insert_text: Some("len($1)$0".to_string()),
        insert_text_format: Some(InsertTextFormat::Snippet as i32),
        documentation: Some(MarkupContent::from_markdown(Builtin::Len.documentation())),
    });
    completions.push(CompletionItem {
        label: "str".to_string(),
        kind: Some(CompletionItemKind::Function as i32),
        insert_text: Some("str($1)$0".to_string()),
        insert_text_format: Some(InsertTextFormat::Snippet as i32),
        documentation: Some(MarkupContent::from_markdown(Builtin::Str.documentation())),
    });
    completions.push(CompletionItem {
        label: "int".to_string(),
        kind: Some(CompletionItemKind::Function as i32),
        insert_text: Some("int($1)$0".to_string()),
        insert_text_format: Some(InsertTextFormat::Snippet as i32),
        documentation: Some(MarkupContent::from_markdown(Builtin::Int.documentation())),
    });
    completions.push(CompletionItem {
        label: "char".to_string(),
        kind: Some(CompletionItemKind::Function as i32),
        insert_text: Some("char($1)$0".to_string()),
        insert_text_format: Some(InsertTextFormat::Snippet as i32),
        documentation: Some(MarkupContent::from_markdown(Builtin::Char.documentation())),
    });
    completions.push(CompletionItem {
        label: "float".to_string(),
        kind: Some(CompletionItemKind::Function as i32),
        insert_text: Some("float($1)$0".to_string()),
        insert_text_format: Some(InsertTextFormat::Snippet as i32),
        documentation: Some(MarkupContent::from_markdown(Builtin::Float.documentation())),
    });
    completions.push(CompletionItem {
        label: "bool".to_string(),
        kind: Some(CompletionItemKind::Function as i32),
        insert_text: Some("bool($1)$0".to_string()),
        insert_text_format: Some(InsertTextFormat::Snippet as i32),
        documentation: Some(MarkupContent::from_markdown(Builtin::Bool.documentation())),
    });
    completions.push(CompletionItem {
        label: "is_null".to_string(),
        kind: Some(CompletionItemKind::Function as i32),
        insert_text: Some("is_null($1)$0".to_string()),
        insert_text_format: Some(InsertTextFormat::Snippet as i32),
        documentation: Some(MarkupContent::from_markdown(
            Builtin::IsNull.documentation(),
        )),
    });
    completions.push(CompletionItem {
        label: "floor".to_string(),
        kind: Some(CompletionItemKind::Function as i32),
        insert_text: Some("floor($1)$0".to_string()),
        insert_text_format: Some(InsertTextFormat::Snippet as i32),
        documentation: Some(MarkupContent::from_markdown(Builtin::Floor.documentation())),
    });
    completions.push(CompletionItem {
        label: "ceil".to_string(),
        kind: Some(CompletionItemKind::Function as i32),
        insert_text: Some("ceil($1)$0".to_string()),
        insert_text_format: Some(InsertTextFormat::Snippet as i32),
        documentation: Some(MarkupContent::from_markdown(Builtin::Ceil.documentation())),
    });
    completions.push(CompletionItem {
        label: "round".to_string(),
        kind: Some(CompletionItemKind::Function as i32),
        insert_text: Some("round($1)$0".to_string()),
        insert_text_format: Some(InsertTextFormat::Snippet as i32),
        documentation: Some(MarkupContent::from_markdown(Builtin::Round.documentation())),
    });
    completions.push(CompletionItem {
        label: "trim_start".to_string(),
        kind: Some(CompletionItemKind::Function as i32),
        insert_text: Some("trim_start($1)$0".to_string()),
        insert_text_format: Some(InsertTextFormat::Snippet as i32),
        documentation: Some(MarkupContent::from_markdown(
            Builtin::TrimStart.documentation(),
        )),
    });
    completions.push(CompletionItem {
        label: "trim_end".to_string(),
        kind: Some(CompletionItemKind::Function as i32),
        insert_text: Some("trim_end($1)$0".to_string()),
        insert_text_format: Some(InsertTextFormat::Snippet as i32),
        documentation: Some(MarkupContent::from_markdown(
            Builtin::TrimEnd.documentation(),
        )),
    });
    completions.push(CompletionItem {
        label: "trim".to_string(),
        kind: Some(CompletionItemKind::Function as i32),
        insert_text: Some("trim($1)$0".to_string()),
        insert_text_format: Some(InsertTextFormat::Snippet as i32),
        documentation: Some(MarkupContent::from_markdown(Builtin::Trim.documentation())),
    });
    completions.push(CompletionItem {
        label: "split".to_string(),
        kind: Some(CompletionItemKind::Function as i32),
        insert_text: Some("split(${1:str}, ${2:delim})$0".to_string()),
        insert_text_format: Some(InsertTextFormat::Snippet as i32),
        documentation: Some(MarkupContent::from_markdown(Builtin::Split.documentation())),
    });
    completions.push(CompletionItem {
        label: "push".to_string(),
        kind: Some(CompletionItemKind::Function as i32),
        insert_text: Some("push(${1:arr}, ${2:value})$0".to_string()),
        insert_text_format: Some(InsertTextFormat::Snippet as i32),
        documentation: Some(MarkupContent::from_markdown(Builtin::Push.documentation())),
    });
    completions.push(CompletionItem {
        label: "pop".to_string(),
        kind: Some(CompletionItemKind::Function as i32),
        insert_text: Some("pop(${1:arr})$0".to_string()),
        insert_text_format: Some(InsertTextFormat::Snippet as i32),
        documentation: Some(MarkupContent::from_markdown(Builtin::Pop.documentation())),
    });
    completions.push(CompletionItem {
        label: "del".to_string(),
        kind: Some(CompletionItemKind::Function as i32),
        insert_text: Some("del(${1:dict}, ${2:key})$0".to_string()),
        insert_text_format: Some(InsertTextFormat::Snippet as i32),
        documentation: Some(MarkupContent::from_markdown(Builtin::Del.documentation())),
    });
    completions.push(CompletionItem {
        label: "print".to_string(),
        kind: Some(CompletionItemKind::Function as i32),
        insert_text: Some("print(${1:value})$0".to_string()),
        insert_text_format: Some(InsertTextFormat::Snippet as i32),
        documentation: Some(MarkupContent::from_markdown(Builtin::Print.documentation())),
    });
    completions.push(CompletionItem {
        label: "input".to_string(),
        kind: Some(CompletionItemKind::Function as i32),
        insert_text: Some("input()$0".to_string()),
        insert_text_format: Some(InsertTextFormat::Snippet as i32),
        documentation: Some(MarkupContent::from_markdown(Builtin::Input.documentation())),
    });
}
