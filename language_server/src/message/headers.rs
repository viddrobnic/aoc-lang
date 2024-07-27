use std::{fmt::Display, io};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Headers {
    pub content_length: usize,
    pub content_type: Option<String>,
}

impl Headers {
    pub fn with_length(content_length: usize) -> Self {
        Self {
            content_length,
            content_type: None,
        }
    }

    pub fn read(input: &mut impl io::BufRead) -> io::Result<Self> {
        let mut headers = Headers {
            content_length: 0,
            content_type: None,
        };

        let mut buf = String::new();
        loop {
            buf.clear();
            input.read_line(&mut buf)?;

            let header = buf.trim_end();
            if header.is_empty() {
                break;
            }

            let Some((name, value)) = header.split_once(": ") else {
                return Err(io::ErrorKind::InvalidData.into());
            };

            if name.eq_ignore_ascii_case("content-length") {
                headers.content_length = value.parse().map_err(|_| io::ErrorKind::InvalidData)?;
            }

            if name.eq_ignore_ascii_case("content-type") {
                headers.content_type = Some(value.to_string());
            }
        }

        Ok(headers)
    }
}

impl Display for Headers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Content-Length: {}\r\n", self.content_length)?;

        if let Some(content_type) = &self.content_type {
            write!(f, "Content-Type: {}\r\n", content_type)?;
        }

        write!(f, "\r\n")
    }
}

#[cfg(test)]
mod test {
    use std::io;

    use super::Headers;

    #[test]
    fn read() -> io::Result<()> {
        let input = "Content-Length: 420\r\n\r\n";
        let mut cursor = io::Cursor::new(input);
        let headers = Headers::read(&mut cursor)?;
        assert_eq!(
            headers,
            Headers {
                content_length: 420,
                content_type: None,
            }
        );

        Ok(())
    }

    #[test]
    fn read_content_type() -> io::Result<()> {
        let input = "Content-Type: test type\r\nContent-Length: 420\r\n\r\n";
        let mut cursor = io::Cursor::new(input);
        let headers = Headers::read(&mut cursor)?;
        assert_eq!(
            headers,
            Headers {
                content_length: 420,
                content_type: Some("test type".to_string()),
            }
        );

        Ok(())
    }

    #[test]
    fn write() {
        let headers = Headers {
            content_length: 6900,
            content_type: None,
        };
        assert_eq!(headers.to_string(), "Content-Length: 6900\r\n\r\n");
    }

    #[test]
    fn write_content_type() {
        let headers = Headers {
            content_length: 6900,
            content_type: Some("hello header!".to_string()),
        };
        assert_eq!(
            headers.to_string(),
            "Content-Length: 6900\r\nContent-Type: hello header!\r\n\r\n"
        );
    }
}
