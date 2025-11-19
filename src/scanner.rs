use crate::definitions::identify_gremlin;
use std::borrow::Cow;

#[derive(Debug, PartialEq, Eq)]
pub struct GremlinLoc {
    pub line: usize,
    pub col: usize,
    pub char_found: char,
    pub description: &'static str,
}

impl GremlinLoc {
    pub fn escape_char(&self) -> String {
        self.char_found.escape_unicode().to_string()
    }
}

pub fn scan_line(line: &str, line_number: usize) -> (Cow<'_, str>, Vec<GremlinLoc>) {
    let mut gremlins = Vec::new();
    let mut output_buffer: Option<String> = None;
    let mut col_num = 1;

    for (idx, c) in line.char_indices() {
        if let Some(action) = identify_gremlin(c) {
            gremlins.push(GremlinLoc {
                line: line_number,
                col: col_num,
                char_found: c,
                description: action.description,
            });

            if output_buffer.is_none() {
                let mut buf = String::with_capacity(line.len());
                buf.push_str(&line[..idx]);
                output_buffer = Some(buf);
            }

            if let Some(buf) = &mut output_buffer {
                if let Some(replacement) = action.replacement {
                    buf.push(replacement);
                }
            }
        } else if let Some(buf) = &mut output_buffer {
            buf.push(c);
        }
        col_num += 1;
    }

    match output_buffer {
        Some(modified) => (Cow::Owned(modified), gremlins),
        None => (Cow::Borrowed(line), gremlins),
    }
}
