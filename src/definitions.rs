#[derive(Debug, Clone, Copy)]
pub struct GremlinAction {
    pub description: &'static str,
    pub replacement: Option<char>,
}

pub fn identify_gremlin(c: char) -> Option<GremlinAction> {
    match c {
        // Trojan Source / Bidi
        '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}' => Some(GremlinAction {
            description: "Bidirectional Text Override (Security Risk!)",
            replacement: None,
        }),

        // Zero Width
        '\u{200B}' => Some(GremlinAction {
            description: "Zero Width Space",
            replacement: None,
        }),
        '\u{200C}' => Some(GremlinAction {
            description: "Zero Width Non-Joiner",
            replacement: None,
        }),
        '\u{200D}' => Some(GremlinAction {
            description: "Zero Width Joiner",
            replacement: None,
        }),
        '\u{FEFF}' => Some(GremlinAction {
            description: "BOM",
            replacement: None,
        }),

        // Weird Spaces
        '\u{00A0}' => Some(GremlinAction {
            description: "NBSP",
            replacement: Some(' '),
        }),
        '\u{2000}'..='\u{200A}' => Some(GremlinAction {
            description: "Variable Width Space",
            replacement: Some(' '),
        }),
        '\u{3000}' => Some(GremlinAction {
            description: "Ideographic Space",
            replacement: Some(' '),
        }),

        // Smart Quotes
        '\u{201C}' | '\u{201D}' => Some(GremlinAction {
            description: "Smart Double Quote",
            replacement: Some('"'),
        }),
        '\u{2018}' | '\u{2019}' => Some(GremlinAction {
            description: "Smart Single Quote",
            replacement: Some('\''),
        }),

        // Homoglyphs
        '\u{037E}' => Some(GremlinAction {
            description: "Greek Question Mark (;)",
            replacement: Some(';'),
        }),

        // General Control (excluding standard whitespace)
        c if c.is_control() && !matches!(c, '\t' | '\n' | '\r') => Some(GremlinAction {
            description: "Non-Whitespace Control Character",
            replacement: None,
        }),

        _ => None,
    }
}
