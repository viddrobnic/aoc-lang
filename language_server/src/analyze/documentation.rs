use parser::ast;

use super::location::{LocationData, LocationEntry};

pub fn make_documentation_location_data(comments: &[ast::Comment]) -> LocationData<String> {
    let mut data = LocationData::default();

    if comments.is_empty() {
        return data;
    }

    let mut block = comments.first().unwrap().clone();
    block.range.start.character = 0;

    for comment in comments.iter().skip(1) {
        if comment.range.start.line - block.range.end.line > 1 {
            data.push(LocationEntry {
                location: block.range,
                entry: block.comment,
            })
            .unwrap();

            block = comment.clone();
            block.range.start.character = 0;

            continue;
        }

        block.comment.push('\n');
        block.comment.push_str(&comment.comment);
        block.range.end = comment.range.end;
    }

    data.push(LocationEntry {
        location: block.range,
        entry: block.comment,
    })
    .unwrap();

    data
}

#[cfg(test)]
mod test {
    use parser::{
        ast::Comment,
        position::{Position, Range},
    };

    use crate::analyze::{
        documentation::make_documentation_location_data,
        location::{LocationData, LocationEntry},
    };

    #[test]
    fn empty() {
        let data = LocationData::default();
        assert_eq!(make_documentation_location_data(&[]), data);
    }

    #[test]
    fn single() {
        let mut data = LocationData::default();
        data.push(LocationEntry {
            location: Range {
                start: Position::new(0, 0),
                end: Position::new(0, 10),
            },
            entry: "foo".to_string(),
        })
        .unwrap();

        assert_eq!(
            make_documentation_location_data(&[Comment {
                comment: "foo".to_string(),
                range: Range {
                    start: Position::new(0, 3),
                    end: Position::new(0, 10)
                }
            }]),
            data
        )
    }

    #[test]
    fn single_block() {
        let mut data = LocationData::default();
        data.push(LocationEntry {
            location: Range {
                start: Position::new(0, 0),
                end: Position::new(2, 10),
            },
            entry: "foo\nbar\nbaz".to_string(),
        })
        .unwrap();

        let input = [
            Comment {
                comment: "foo".to_string(),
                range: Range {
                    start: Position::new(0, 3),
                    end: Position::new(0, 8),
                },
            },
            Comment {
                comment: "bar".to_string(),
                range: Range {
                    start: Position::new(1, 5),
                    end: Position::new(1, 10),
                },
            },
            Comment {
                comment: "baz".to_string(),
                range: Range {
                    start: Position::new(2, 5),
                    end: Position::new(2, 10),
                },
            },
        ];

        assert_eq!(make_documentation_location_data(&input), data)
    }

    #[test]
    fn multiple_blocks() {
        let mut data = LocationData::default();
        data.push(LocationEntry {
            location: Range {
                start: Position::new(0, 0),
                end: Position::new(1, 10),
            },
            entry: "foo\nbar".to_string(),
        })
        .unwrap();
        data.push(LocationEntry {
            location: Range {
                start: Position::new(4, 0),
                end: Position::new(4, 12),
            },
            entry: "baz".to_string(),
        })
        .unwrap();

        let input = [
            Comment {
                comment: "foo".to_string(),
                range: Range {
                    start: Position::new(0, 3),
                    end: Position::new(0, 8),
                },
            },
            Comment {
                comment: "bar".to_string(),
                range: Range {
                    start: Position::new(1, 5),
                    end: Position::new(1, 10),
                },
            },
            Comment {
                comment: "baz".to_string(),
                range: Range {
                    start: Position::new(4, 5),
                    end: Position::new(4, 12),
                },
            },
        ];

        assert_eq!(make_documentation_location_data(&input), data)
    }
}
