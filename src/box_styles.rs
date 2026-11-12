#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Box {
    // top
    pub top_left: char,
    pub top: char,
    pub top_divider: char,
    pub top_right: char,
    // head
    pub head_left: char,
    pub head_vertical: char,
    pub head_right: char,
    // head_row
    pub head_row_left: char,
    pub head_row_horizontal: char,
    pub head_row_cross: char,
    pub head_row_right: char,
    // mid
    pub mid_left: char,
    pub mid_vertical: char,
    pub mid_right: char,
    // row
    pub row_left: char,
    pub row_horizontal: char,
    pub row_cross: char,
    pub row_right: char,
    // foot_row
    pub foot_row_left: char,
    pub foot_row_horizontal: char,
    pub foot_row_cross: char,
    pub foot_row_right: char,
    // foot
    pub foot_left: char,
    pub foot_vertical: char,
    pub foot_right: char,
    // bottom
    pub bottom_left: char,
    pub bottom: char,
    pub bottom_divider: char,
    pub bottom_right: char,
}

impl Box {
    pub const fn from_chars(chars: [char; 32]) -> Self {
        Self {
            top_left: chars[0],
            top: chars[1],
            top_divider: chars[2],
            top_right: chars[3],

            head_left: chars[4],
            head_vertical: chars[6],
            head_right: chars[7],

            head_row_left: chars[8],
            head_row_horizontal: chars[9],
            head_row_cross: chars[10],
            head_row_right: chars[11],

            mid_left: chars[12],
            mid_vertical: chars[14],
            mid_right: chars[15],

            row_left: chars[16],
            row_horizontal: chars[17],
            row_cross: chars[18],
            row_right: chars[19],

            foot_row_left: chars[20],
            foot_row_horizontal: chars[21],
            foot_row_cross: chars[22],
            foot_row_right: chars[23],

            foot_left: chars[24],
            foot_vertical: chars[26],
            foot_right: chars[27],

            bottom_left: chars[28],
            bottom: chars[29],
            bottom_divider: chars[30],
            bottom_right: chars[31],
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        s.parse().expect("Invalid box string")
    }

    pub fn from_box_str(s: &str) -> Self {
        s.parse().expect("Invalid box string")
    }
}

impl std::str::FromStr for Box {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lines: Vec<&str> = s.lines().collect();
        if lines.len() != 8 {
            return Err("Box string must contain 8 lines");
        }
        let mut chars = [' '; 32];
        for (row, line) in lines.iter().enumerate() {
            let row_chars: Vec<char> = line.chars().collect();
            if row_chars.len() < 4 {
                return Err("Each line in box must have at least 4 characters");
            }
            for col in 0..4 {
                chars[row * 4 + col] = row_chars[col];
            }
        }
        Ok(Self::from_chars(chars))
    }
}

impl Box {
    pub fn get_top(&self, widths: &[usize]) -> String {
        let mut out = String::new();
        out.push(self.top_left);
        for (i, &w) in widths.iter().enumerate() {
            out.push_str(&self.top.to_string().repeat(w));
            if i + 1 < widths.len() {
                out.push(self.top_divider);
            }
        }
        out.push(self.top_right);
        out
    }

    pub fn get_row(&self, widths: &[usize], level: &str) -> String {
        let (left, horiz, cross, right) = match level {
            "head" => (
                self.head_row_left,
                self.head_row_horizontal,
                self.head_row_cross,
                self.head_row_right,
            ),
            "foot" => (
                self.foot_row_left,
                self.foot_row_horizontal,
                self.foot_row_cross,
                self.foot_row_right,
            ),
            "mid" => (
                self.mid_left,
                self.row_horizontal,
                self.mid_vertical,
                self.mid_right,
            ),
            _ => (
                self.row_left,
                self.row_horizontal,
                self.row_cross,
                self.row_right,
            ),
        };

        let mut out = String::new();
        out.push(left);
        for (i, &w) in widths.iter().enumerate() {
            out.push_str(&horiz.to_string().repeat(w));
            if i + 1 < widths.len() {
                out.push(cross);
            }
        }
        out.push(right);
        out
    }

    pub fn get_bottom(&self, widths: &[usize]) -> String {
        let mut out = String::new();
        out.push(self.bottom_left);
        for (i, &w) in widths.iter().enumerate() {
            out.push_str(&self.bottom.to_string().repeat(w));
            if i + 1 < widths.len() {
                out.push(self.bottom_divider);
            }
        }
        out.push(self.bottom_right);
        out
    }
}

pub static ASCII: Box = Box::from_chars([
    '+', '-', '-', '+', '|', ' ', '|', '|', '|', '-', '+', '|', '|', ' ', '|', '|', '|', '-', '+',
    '|', '|', '-', '+', '|', '|', ' ', '|', '|', '+', '-', '-', '+',
]);

pub static ASCII2: Box = Box::from_chars([
    '+', '-', '+', '+', '|', ' ', '|', '|', '+', '-', '+', '+', '|', ' ', '|', '|', '+', '-', '+',
    '+', '+', '-', '+', '+', '|', ' ', '|', '|', '+', '-', '+', '+',
]);

pub static SQUARE: Box = Box::from_chars([
    '┌', '─', '┬', '┐', '│', ' ', '│', '│', '├', '─', '┼', '┤', '│', ' ', '│', '│', '├', '─', '┼',
    '┤', '├', '─', '┼', '┤', '│', ' ', '│', '│', '└', '─', '┴', '┘',
]);

pub static SQUARE_DOUBLE_HEAD: Box = Box::from_chars([
    '┌', '─', '┬', '┐', '│', ' ', '│', '│', '╞', '═', '╪', '╡', '│', ' ', '│', '│', '├', '─', '┼',
    '┤', '├', '─', '┼', '┤', '│', ' ', '│', '│', '└', '─', '┴', '┘',
]);

pub static ROUNDED: Box = Box::from_chars([
    '╭', '─', '┬', '╮', '│', ' ', '│', '│', '├', '─', '┼', '┤', '│', ' ', '│', '│', '├', '─', '┼',
    '┤', '├', '─', '┼', '┤', '│', ' ', '│', '│', '╰', '─', '┴', '╯',
]);

pub static HEAVY: Box = Box::from_chars([
    '┏', '━', '┳', '┓', '┃', ' ', '┃', '┃', '┣', '━', '╋', '┫', '┃', ' ', '┃', '┃', '┣', '━', '╋',
    '┫', '┣', '━', '╋', '┫', '┃', ' ', '┃', '┃', '┗', '━', '┻', '┛',
]);

pub static HEAVY_HEAD: Box = Box::from_chars([
    '┏', '━', '┳', '┓', '┃', ' ', '┃', '┃', '┡', '━', '╇', '┩', '│', ' ', '│', '│', '├', '─', '┼',
    '┤', '├', '─', '┼', '┤', '│', ' ', '│', '│', '└', '─', '┴', '┘',
]);

pub static DOUBLE: Box = Box::from_chars([
    '╔', '═', '╦', '╗', '║', ' ', '║', '║', '╠', '═', '╬', '╣', '║', ' ', '║', '║', '╠', '═', '╬',
    '╣', '╠', '═', '╬', '╣', '║', ' ', '║', '║', '╚', '═', '╩', '╝',
]);

pub static DOUBLE_EDGE: Box = Box::from_chars([
    '╔', '═', '╤', '╗', '║', ' ', '│', '║', '╠', '═', '╪', '╣', '║', ' ', '│', '║', '╟', '─', '┼',
    '╢', '╠', '═', '╪', '╣', '║', ' ', '│', '║', '╚', '═', '╧', '╝',
]);

pub static MINIMAL: Box = Box::from_chars([
    ' ', ' ', '╷', ' ', ' ', ' ', '│', ' ', '╶', '─', '┼', '╴', ' ', ' ', '│', ' ', '╶', '─', '┼',
    '╴', '╶', '─', '┼', '╴', ' ', ' ', '│', ' ', ' ', ' ', '╵', ' ',
]);

pub static SIMPLE: Box = Box::from_chars([
    ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '─', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
    ' ', ' ', '─', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
]);

pub static HORIZONTALS: Box = Box::from_chars([
    ' ', '─', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '─', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '─', ' ',
    ' ', ' ', '─', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '─', ' ', ' ',
]);

pub static MARKDOWN: Box = Box::from_chars([
    ' ', ' ', ' ', ' ', '|', ' ', '|', '|', '|', '-', '+', '|', '|', ' ', '|', '|', '|', ' ', '|',
    '|', '|', '-', '+', '|', '|', ' ', '|', '|', ' ', ' ', ' ', ' ',
]);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rounded_top_and_bottom() {
        let top = ROUNDED.get_top(&[5, 10]);
        assert_eq!(top, "╭─────┬──────────╮");

        let bottom = ROUNDED.get_bottom(&[5, 10]);
        assert_eq!(bottom, "╰─────┴──────────╯");

        let row = ROUNDED.get_row(&[5, 10], "head");
        assert_eq!(row, "├─────┼──────────┤");
    }
}
