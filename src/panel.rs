use crate::align::AlignMethod;
use crate::box_styles::{Box as BoxStyle, ROUNDED};
use crate::console::Console;
use crate::markup::render as render_markup;
use crate::measure::Measurement;
use crate::padding::Padding;
use crate::protocol::{ConsoleOptions, Renderable};
use crate::segment::Segment;
use crate::style::Style;
use crate::text::Text;
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct Panel<T> {
    pub renderable: T,
    pub box_style: BoxStyle,
    pub title: Option<Text>,
    pub title_align: AlignMethod,
    pub subtitle: Option<Text>,
    pub subtitle_align: AlignMethod,
    pub border_style: Option<Style>,
    pub style: Option<Style>,
    pub expand: bool,
    pub width: Option<usize>,
    pub height: Option<usize>,
    pub padding: (usize, usize), // (vertical, horizontal)
    pub margin: (usize, usize),  // (vertical, horizontal)
    pub border_top: bool,
    pub border_bottom: bool,
    pub border_left: bool,
    pub border_right: bool,
}

impl<T> Panel<T> {
    pub fn new(renderable: T) -> Self {
        Self {
            renderable,
            box_style: ROUNDED,
            title: None,
            title_align: AlignMethod::Center,
            subtitle: None,
            subtitle_align: AlignMethod::Center,
            border_style: None,
            style: None,
            expand: true,
            width: None,
            height: None,
            padding: (0, 1),
            margin: (0, 0),
            border_top: true,
            border_bottom: true,
            border_left: true,
            border_right: true,
        }
    }

    pub fn fit(renderable: T) -> Self {
        let mut p = Self::new(renderable);
        p.expand = false;
        p
    }

    pub fn box_style(mut self, box_style: BoxStyle) -> Self {
        self.box_style = box_style;
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        let s = title.into();
        self.title = Some(render_markup(&s).unwrap_or_else(|_| Text::from_plain(s)));
        self
    }

    pub fn title_text(mut self, text: Text) -> Self {
        self.title = Some(text);
        self
    }

    pub fn title_align(mut self, align: AlignMethod) -> Self {
        self.title_align = align;
        self
    }

    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        let s = subtitle.into();
        self.subtitle = Some(render_markup(&s).unwrap_or_else(|_| Text::from_plain(s)));
        self
    }

    pub fn subtitle_text(mut self, text: Text) -> Self {
        self.subtitle = Some(text);
        self
    }

    pub fn subtitle_align(mut self, align: AlignMethod) -> Self {
        self.subtitle_align = align;
        self
    }

    pub fn border_style(mut self, style: Style) -> Self {
        self.border_style = Some(style);
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    pub fn expand(mut self, expand: bool) -> Self {
        self.expand = expand;
        self
    }

    pub fn width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: usize) -> Self {
        self.height = Some(height);
        self
    }

    pub fn padding(mut self, pad_v: usize, pad_h: usize) -> Self {
        self.padding = (pad_v, pad_h);
        self
    }

    pub fn margin(mut self, mar_v: usize, mar_h: usize) -> Self {
        self.margin = (mar_v, mar_h);
        self
    }

    pub fn border_sides(mut self, top: bool, right: bool, bottom: bool, left: bool) -> Self {
        self.border_top = top;
        self.border_right = right;
        self.border_bottom = bottom;
        self.border_left = left;
        self
    }

    pub fn left_accent(mut self) -> Self {
        self.border_top = false;
        self.border_bottom = false;
        self.border_right = false;
        self.border_left = true;
        self
    }

    pub fn horizontal_only(mut self) -> Self {
        self.border_top = true;
        self.border_bottom = true;
        self.border_left = false;
        self.border_right = false;
        self
    }
}

impl Panel<Text> {
    pub fn from_markup(content: &str) -> Self {
        let text = render_markup(content).unwrap_or_else(|_| Text::from_plain(content));
        Panel::new(text)
    }
}

impl<T: Renderable> Renderable for Panel<T> {
    fn render_to<'a>(
        &'a self,
        console: &Console,
        options: &ConsoleOptions,
        sink: &mut Vec<Segment<'a>>,
    ) {
        let max_width = self.width.unwrap_or(options.max_width);
        let border_box = self.box_style;
        let border_style = self.border_style.clone();

        let left_w = if self.border_left { 1 } else { 0 };
        let right_w = if self.border_right { 1 } else { 0 };
        let border_w = left_w + right_w;
        let margin_h = self.margin.1 * 2;
        let avail_w = max_width.saturating_sub(margin_h);

        // Calculate inner width
        let child_width = if self.expand {
            avail_w.saturating_sub(border_w)
        } else {
            let measured = self
                .renderable
                .measure(console, &options.update_width(avail_w.saturating_sub(border_w)));
            let pad_h = self.padding.1 * 2;
            let mut w = measured.maximum + pad_h;
            if let Some(ref t) = self.title {
                w = w.max(t.cell_len() + 2);
            }
            if let Some(ref st) = self.subtitle {
                w = w.max(st.cell_len() + 2);
            }
            w.min(avail_w.saturating_sub(border_w))
        };

        let panel_width = child_width + border_w;
        let pad_v = self.padding.0;
        let pad_h = self.padding.1;

        let margin_left_seg = if self.margin.1 > 0 {
            Some(Segment::unstyled(Cow::Owned(" ".repeat(self.margin.1))))
        } else {
            None
        };

        // Top margin
        for _ in 0..self.margin.0 {
            sink.push(Segment::line());
        }

        // Top border with optional title
        if self.border_top {
            if let Some(ref margin_seg) = margin_left_seg {
                sink.push(margin_seg.clone());
            }

            if let Some(ref title) = self.title {
                if panel_width > 4 {
                    let avail_title_space = panel_width.saturating_sub(4);
                    let title_len = title.cell_len() + 2; // +2 for surrounding spaces
                    let excess = avail_title_space.saturating_sub(title_len);
                    let (left_pad, right_pad) = match self.title_align {
                        AlignMethod::Left => (0, excess),
                        AlignMethod::Right => (excess, 0),
                        AlignMethod::Center => {
                            let l = excess / 2;
                            (l, excess - l)
                        }
                    };

                    sink.push(Segment {
                        text: Cow::Owned(format!(
                            "{}{}",
                            if self.border_left { border_box.top_left.to_string() } else { "".to_string() },
                            border_box.top.to_string().repeat(left_pad + 1)
                        )),
                        style: border_style.clone(),
                        control: None,
                    });
                    sink.push(Segment::unstyled(Cow::Borrowed(" ")));

                    sink.extend(title.render_to_segments());

                    sink.push(Segment::unstyled(Cow::Borrowed(" ")));
                    sink.push(Segment {
                        text: Cow::Owned(format!(
                            "{}{}",
                            border_box.top.to_string().repeat(right_pad + 1),
                            if self.border_right { border_box.top_right.to_string() } else { "".to_string() }
                        )),
                        style: border_style.clone(),
                        control: None,
                    });
                } else {
                    sink.push(Segment {
                        text: Cow::Owned(border_box.get_top(&[child_width])),
                        style: border_style.clone(),
                        control: None,
                    });
                }
            } else {
                let top_str = if self.border_left && self.border_right {
                    border_box.get_top(&[child_width])
                } else {
                    format!(
                        "{}{}{}",
                        if self.border_left { border_box.top_left.to_string() } else { "".to_string() },
                        border_box.top.to_string().repeat(child_width),
                        if self.border_right { border_box.top_right.to_string() } else { "".to_string() }
                    )
                };
                sink.push(Segment {
                    text: Cow::Owned(top_str),
                    style: border_style.clone(),
                    control: None,
                });
            }
            sink.push(Segment::line());
        } else if let Some(ref title) = self.title {
            if let Some(ref margin_seg) = margin_left_seg {
                sink.push(margin_seg.clone());
            }
            sink.extend(title.render_to_segments());
            sink.push(Segment::line());
        }

        // Body lines wrapped in padding
        let padded = Padding::new(&self.renderable, (pad_v, pad_h, pad_v, pad_h));
        let child_opts = options.update_width(child_width);
        let lines = console.render_lines_owned(&padded, Some(&child_opts), true);

        for line in lines {
            if let Some(ref margin_seg) = margin_left_seg {
                sink.push(margin_seg.clone());
            }

            if self.border_left {
                sink.push(Segment {
                    text: Cow::Owned(border_box.mid_left.to_string()),
                    style: border_style.clone(),
                    control: None,
                });
            }
            sink.extend(line);
            if self.border_right {
                sink.push(Segment {
                    text: Cow::Owned(border_box.mid_right.to_string()),
                    style: border_style.clone(),
                    control: None,
                });
            }
            sink.push(Segment::line());
        }

        // Bottom border with optional subtitle
        if self.border_bottom {
            if let Some(ref margin_seg) = margin_left_seg {
                sink.push(margin_seg.clone());
            }

            if let Some(ref subtitle) = self.subtitle {
                if panel_width > 4 {
                    let avail_space = panel_width.saturating_sub(4);
                    let sub_len = subtitle.cell_len() + 2; // +2 for surrounding spaces
                    let excess = avail_space.saturating_sub(sub_len);
                    let (left_pad, right_pad) = match self.subtitle_align {
                        AlignMethod::Left => (0, excess),
                        AlignMethod::Right => (excess, 0),
                        AlignMethod::Center => {
                            let l = excess / 2;
                            (l, excess - l)
                        }
                    };

                    sink.push(Segment {
                        text: Cow::Owned(format!(
                            "{}{}",
                            if self.border_left { border_box.bottom_left.to_string() } else { "".to_string() },
                            border_box.bottom.to_string().repeat(left_pad + 1)
                        )),
                        style: border_style.clone(),
                        control: None,
                    });
                    sink.push(Segment::unstyled(Cow::Borrowed(" ")));

                    sink.extend(subtitle.render_to_segments());

                    sink.push(Segment::unstyled(Cow::Borrowed(" ")));
                    sink.push(Segment {
                        text: Cow::Owned(format!(
                            "{}{}",
                            border_box.bottom.to_string().repeat(right_pad + 1),
                            if self.border_right { border_box.bottom_right.to_string() } else { "".to_string() }
                        )),
                        style: border_style,
                        control: None,
                    });
                } else {
                    sink.push(Segment {
                        text: Cow::Owned(border_box.get_bottom(&[child_width])),
                        style: border_style,
                        control: None,
                    });
                }
            } else {
                let bottom_str = if self.border_left && self.border_right {
                    border_box.get_bottom(&[child_width])
                } else {
                    format!(
                        "{}{}{}",
                        if self.border_left { border_box.bottom_left.to_string() } else { "".to_string() },
                        border_box.bottom.to_string().repeat(child_width),
                        if self.border_right { border_box.bottom_right.to_string() } else { "".to_string() }
                    )
                };
                sink.push(Segment {
                    text: Cow::Owned(bottom_str),
                    style: border_style,
                    control: None,
                });
            }
        } else if let Some(ref subtitle) = self.subtitle {
            if let Some(ref margin_seg) = margin_left_seg {
                sink.push(margin_seg.clone());
            }
            sink.extend(subtitle.render_to_segments());
        }

        // Bottom margin
        for _ in 0..self.margin.0 {
            sink.push(Segment::line());
        }
    }

    fn measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
        let inner = self.renderable.measure(console, options);
        let left_w = if self.border_left { 1 } else { 0 };
        let right_w = if self.border_right { 1 } else { 0 };
        let pad_h = self.padding.1 * 2 + left_w + right_w + self.margin.1 * 2;
        let mut min = inner.minimum + pad_h;
        let mut max = inner.maximum + pad_h;
        if let Some(ref t) = self.title {
            let t_len = t.cell_len() + 4;
            min = min.max(t_len);
            max = max.max(t_len);
        }
        Measurement::new(min, max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::box_styles::ASCII;
    use crate::color::ColorSystem;

    #[test]
    fn test_panel_simple() {
        let console = Console::with_dimensions(15, 5).with_color_system(ColorSystem::NoColor);
        let panel = Panel::new("Hello").box_style(ASCII).padding(0, 1);
        let out = console.capture(|c| {
            c.print(panel);
        });

        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines[0], "+-------------+");
        assert_eq!(lines[1], "| Hello       |");
        assert_eq!(lines[2], "+-------------+");
    }

    #[test]
    fn test_panel_left_accent() {
        let console = Console::with_dimensions(15, 5).with_color_system(ColorSystem::NoColor);
        let panel = Panel::new("Note").box_style(ASCII).left_accent().padding(0, 1);
        let out = console.capture(|c| {
            c.print(panel);
        });

        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 1);
        assert!(lines[0].starts_with("| Note"));
        assert!(!lines[0].ends_with("|"));
    }

    #[test]
    fn test_panel_margin() {
        let console = Console::with_dimensions(20, 5).with_color_system(ColorSystem::NoColor);
        let panel = Panel::new("Text").box_style(ASCII).margin(1, 2).padding(0, 0);
        let out = console.capture(|c| {
            c.print(panel);
        });

        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines[0], ""); // vertical margin
        assert!(lines[1].starts_with("  +")); // horizontal margin 2 spaces
    }
}
