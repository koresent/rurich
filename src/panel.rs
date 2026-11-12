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

        // Calculate inner width
        let child_width = if self.expand {
            max_width.saturating_sub(2)
        } else {
            let measured = self
                .renderable
                .measure(console, &options.update_width(max_width.saturating_sub(2)));
            let pad_h = self.padding.1 * 2;
            let mut w = measured.maximum + pad_h;
            if let Some(ref t) = self.title {
                w = w.max(t.cell_len() + 2);
            }
            if let Some(ref st) = self.subtitle {
                w = w.max(st.cell_len() + 2);
            }
            w.min(max_width.saturating_sub(2))
        };

        let panel_width = child_width + 2;
        let pad_v = self.padding.0;
        let pad_h = self.padding.1;

        // Top border with optional title
        if let Some(ref title) = self.title {
            if panel_width > 4 {
                let avail_title_space = panel_width - 4;
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
                        border_box.top_left,
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
                        border_box.top_right
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
            sink.push(Segment {
                text: Cow::Owned(border_box.get_top(&[child_width])),
                style: border_style.clone(),
                control: None,
            });
        }
        sink.push(Segment::line());

        // Body lines wrapped in padding
        let padded = Padding::new(&self.renderable, (pad_v, pad_h, pad_v, pad_h));
        let child_opts = options.update_width(child_width);
        let lines = console.render_lines_owned(&padded, Some(&child_opts), true);

        for line in lines {
            sink.push(Segment {
                text: Cow::Owned(border_box.mid_left.to_string()),
                style: border_style.clone(),
                control: None,
            });
            sink.extend(line);
            sink.push(Segment {
                text: Cow::Owned(border_box.mid_right.to_string()),
                style: border_style.clone(),
                control: None,
            });
            sink.push(Segment::line());
        }

        // Bottom border with optional subtitle
        if let Some(ref subtitle) = self.subtitle {
            if panel_width > 4 {
                let avail_space = panel_width - 4;
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
                        border_box.bottom_left,
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
                        border_box.bottom_right
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
            sink.push(Segment {
                text: Cow::Owned(border_box.get_bottom(&[child_width])),
                style: border_style,
                control: None,
            });
        }
    }

    fn measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
        let inner = self.renderable.measure(console, options);
        let pad_h = self.padding.1 * 2 + 2; // +2 for borders
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
    fn test_panel_title() {
        let console = Console::with_dimensions(15, 5).with_color_system(ColorSystem::NoColor);
        let panel = Panel::new("Content")
            .box_style(ASCII)
            .title("Title")
            .title_align(AlignMethod::Center)
            .padding(0, 0);
        let out = console.capture(|c| {
            c.print(panel);
        });

        let lines: Vec<&str> = out.lines().collect();
        assert!(lines[0].contains("Title"));
        assert!(lines[1].contains("Content"));
    }
}
