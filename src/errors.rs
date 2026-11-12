use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RurichError {
    MarkupError(String),
    StyleSyntaxError(String),
    ColorParseError(String),
    RenderError(String),
}

impl fmt::Display for RurichError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MarkupError(msg) => write!(f, "Markup error: {msg}"),
            Self::StyleSyntaxError(msg) => write!(f, "Style syntax error: {msg}"),
            Self::ColorParseError(msg) => write!(f, "Color parse error: {msg}"),
            Self::RenderError(msg) => write!(f, "Render error: {msg}"),
        }
    }
}

impl std::error::Error for RurichError {}
