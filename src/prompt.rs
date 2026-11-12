use crate::markup::render as render_markup;
use crate::segment::Segment;
use std::error::Error;
use std::fmt::{self, Display};
use std::io::{self, BufRead, Write};
use std::str::FromStr;

/// Error returned when prompt input is invalid.
#[derive(Debug)]
pub enum PromptError {
    Io(io::Error),
    InvalidValue(String),
}

impl Display for PromptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PromptError::Io(err) => write!(f, "IO error: {err}"),
            PromptError::InvalidValue(val) => write!(f, "Invalid value entered: {val}"),
        }
    }
}

impl Error for PromptError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            PromptError::Io(err) => Some(err),
            PromptError::InvalidValue(_) => None,
        }
    }
}

impl From<io::Error> for PromptError {
    fn from(err: io::Error) -> Self {
        PromptError::Io(err)
    }
}

/// Interactive terminal prompt with markup support, validation, choices, and default values.
pub struct Prompt<T = String> {
    pub prompt: String,
    pub default: Option<T>,
    pub choices: Vec<String>,
    pub show_default: bool,
    pub show_choices: bool,
}

impl<T: FromStr + Display + Clone> Prompt<T> {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            default: None,
            choices: Vec::new(),
            show_default: true,
            show_choices: true,
        }
    }

    pub fn default(mut self, default: T) -> Self {
        self.default = Some(default);
        self
    }

    pub fn choices<I, S>(mut self, choices: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.choices = choices.into_iter().map(Into::into).collect();
        self
    }

    pub fn show_default(mut self, show: bool) -> Self {
        self.show_default = show;
        self
    }

    pub fn show_choices(mut self, show: bool) -> Self {
        self.show_choices = show;
        self
    }

    /// Ask the user for input via standard I/O.
    pub fn ask(&self) -> Result<T, PromptError> {
        let stdin = io::stdin();
        let mut reader = stdin.lock();
        let mut stdout = io::stdout();
        self.ask_stream(&mut reader, &mut stdout)
    }

    /// Ask using provided reader and writer streams (useful for testing and redirection).
    pub fn ask_stream<R: BufRead, W: Write>(
        &self,
        reader: &mut R,
        writer: &mut W,
    ) -> Result<T, PromptError> {
        loop {
            // Build prompt string
            let mut prompt_markup = self.prompt.clone();

            if self.show_choices && !self.choices.is_empty() {
                prompt_markup.push_str(&format!(" [yellow][{}][/]", self.choices.join("/")));
            }

            if self.show_default
                && let Some(ref def) = self.default
            {
                prompt_markup.push_str(&format!(" [dim]({def})[/]"));
            }

            prompt_markup.push_str(": ");

            let text = render_markup(&prompt_markup).map_err(|e| {
                PromptError::Io(io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
            })?;

            let segs = text.render_to_segments();
            let ansi_prompt = Segment::render_ansi(&segs, crate::color::ColorSystem::TrueColor);
            writer.write_all(ansi_prompt.as_bytes())?;
            writer.flush()?;

            let mut input = String::new();
            let bytes_read = reader.read_line(&mut input)?;
            if bytes_read == 0 && self.default.is_none() {
                return Err(PromptError::Io(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "End of input",
                )));
            }

            let trimmed = input.trim();
            if trimmed.is_empty()
                && let Some(ref def) = self.default
            {
                return Ok(def.clone());
            }

            if !self.choices.is_empty()
                && !self.choices.iter().any(|c| c.eq_ignore_ascii_case(trimmed))
            {
                writeln!(
                    writer,
                    "\x1b[31mPlease enter one of: {}\x1b[0m",
                    self.choices.join(", ")
                )?;
                continue;
            }

            match trimmed.parse::<T>() {
                Ok(val) => return Ok(val),
                Err(_) => {
                    writeln!(writer, "\x1b[31mPlease enter a valid value.\x1b[0m")?;
                }
            }
        }
    }
}

/// Confirmation prompt expecting boolean input (y/n, yes/no, true/false).
pub struct Confirm {
    prompt: String,
    default: Option<bool>,
}

impl Confirm {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            default: None,
        }
    }

    pub fn default(mut self, default: bool) -> Self {
        self.default = Some(default);
        self
    }

    pub fn ask(&self) -> Result<bool, PromptError> {
        let stdin = io::stdin();
        let mut reader = stdin.lock();
        let mut stdout = io::stdout();
        self.ask_stream(&mut reader, &mut stdout)
    }

    pub fn ask_stream<R: BufRead, W: Write>(
        &self,
        reader: &mut R,
        writer: &mut W,
    ) -> Result<bool, PromptError> {
        let choices_label = match self.default {
            Some(true) => "[Y/n]",
            Some(false) => "[y/N]",
            None => "[y/n]",
        };

        loop {
            let prompt_markup = format!("{} [yellow]{}[/]: ", self.prompt, choices_label);
            let text = render_markup(&prompt_markup).map_err(|e| {
                PromptError::Io(io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
            })?;

            let segs = text.render_to_segments();
            let ansi_prompt = Segment::render_ansi(&segs, crate::color::ColorSystem::TrueColor);
            writer.write_all(ansi_prompt.as_bytes())?;
            writer.flush()?;

            let mut input = String::new();
            let bytes_read = reader.read_line(&mut input)?;
            if bytes_read == 0 && self.default.is_none() {
                return Err(PromptError::Io(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "End of input",
                )));
            }

            let trimmed = input.trim().to_lowercase();
            if trimmed.is_empty()
                && let Some(def) = self.default
            {
                return Ok(def);
            }

            match trimmed.as_str() {
                "y" | "yes" | "t" | "true" | "1" => return Ok(true),
                "n" | "no" | "f" | "false" | "0" => return Ok(false),
                _ => {
                    writeln!(writer, "\x1b[31mPlease enter y or n.\x1b[0m")?;
                }
            }
        }
    }
}

pub type IntPrompt = Prompt<i64>;
pub type FloatPrompt = Prompt<f64>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_prompt_default_value() {
        let prompt: Prompt<String> = Prompt::new("Name").default("Alice".into());
        let mut input = Cursor::new(b"\n");
        let mut output = Vec::new();

        let res = prompt.ask_stream(&mut input, &mut output).unwrap();
        assert_eq!(res, "Alice");
    }

    #[test]
    fn test_prompt_with_input() {
        let prompt: Prompt<String> = Prompt::new("City");
        let mut input = Cursor::new(b"London\n");
        let mut output = Vec::new();

        let res = prompt.ask_stream(&mut input, &mut output).unwrap();
        assert_eq!(res, "London");
    }

    #[test]
    fn test_prompt_int() {
        let prompt: Prompt<i32> = Prompt::new("Age");
        let mut input = Cursor::new(b"25\n");
        let mut output = Vec::new();

        let res = prompt.ask_stream(&mut input, &mut output).unwrap();
        assert_eq!(res, 25);
    }

    #[test]
    fn test_confirm_default_yes() {
        let confirm = Confirm::new("Proceed?").default(true);
        let mut input = Cursor::new(b"\n");
        let mut output = Vec::new();

        let res = confirm.ask_stream(&mut input, &mut output).unwrap();
        assert!(res);
    }

    #[test]
    fn test_confirm_explicit_no() {
        let confirm = Confirm::new("Delete file?");
        let mut input = Cursor::new(b"no\n");
        let mut output = Vec::new();

        let res = confirm.ask_stream(&mut input, &mut output).unwrap();
        assert!(!res);
    }
}
