use crate::console::Console;
use crate::markup::render as render_markup;
use crate::measure::Measurement;
use crate::protocol::{ConsoleOptions, Renderable};
use crate::segment::Segment;
use crate::style::Style;
use crate::text::Text;
use std::borrow::Cow;

pub type TreeGuides = (&'static str, &'static str, &'static str, &'static str);

pub const DEFAULT_GUIDES: TreeGuides = ("    ", "│   ", "├── ", "└── ");
pub const BOLD_GUIDES: TreeGuides = ("    ", "┃   ", "┣━━ ", "┗━━ ");
pub const ASCII_GUIDES: TreeGuides = ("    ", "|   ", "+-- ", "`-- ");
pub const DOUBLE_GUIDES: TreeGuides = ("    ", "║   ", "╠══ ", "╚══ ");

pub struct Tree {
    pub label: Box<dyn Renderable>,
    pub children: Vec<Tree>,
    pub guides: TreeGuides,
    pub guide_style: Option<Style>,
    pub style: Option<Style>,
    pub expanded: bool,
    pub hide_root: bool,
}

impl Tree {
    pub fn new(label: impl Renderable + 'static) -> Self {
        Self {
            label: Box::new(label),
            children: Vec::new(),
            guides: DEFAULT_GUIDES,
            guide_style: Some(Style::parse("tree.line").unwrap_or_default()),
            style: None,
            expanded: true,
            hide_root: false,
        }
    }

    pub fn from_markup(label: &str) -> Self {
        let text = render_markup(label).unwrap_or_else(|_| Text::from_plain(label));
        Self::new(text)
    }

    pub fn guides(mut self, guides: TreeGuides) -> Self {
        self.guides = guides;
        self
    }

    pub fn guide_style(mut self, style: Style) -> Self {
        self.guide_style = Some(style);
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    pub fn hide_root(mut self, hide: bool) -> Self {
        self.hide_root = hide;
        self
    }

    pub fn add(&mut self, label: impl Renderable + 'static) -> &mut Tree {
        let mut child = Tree::new(label);
        child.guides = self.guides;
        child.guide_style = self.guide_style.clone();
        child.style = self.style.clone();
        self.children.push(child);
        self.children.last_mut().unwrap()
    }

    pub fn add_markup(&mut self, label: &str) -> &mut Tree {
        let text = render_markup(label).unwrap_or_else(|_| Text::from_plain(label));
        self.add(text)
    }

    pub fn add_tree(&mut self, child: Tree) -> &mut Tree {
        self.children.push(child);
        self.children.last_mut().unwrap()
    }

    /// Render tree recursively into a list of lines of Segments
    fn render_node<'a>(
        &'a self,
        console: &Console,
        options: &ConsoleOptions,
        prefixes: &[(bool, Style)], // (is_last_at_level, guide_style)
        is_last: bool,
        is_root: bool,
        sink: &mut Vec<Segment<'a>>,
    ) {
        let (space_guide, cont_guide, fork_guide, end_guide) = self.guides;
        let g_style = self.guide_style.clone().unwrap_or_default();

        if !is_root || !self.hide_root {
            // Build prefix for the first line of this node
            if !is_root {
                for &(ancestor_last, ref ancestor_style) in prefixes {
                    let text = if ancestor_last {
                        space_guide
                    } else {
                        cont_guide
                    };
                    sink.push(Segment {
                        text: Cow::Borrowed(text),
                        style: Some(ancestor_style.clone()),
                        control: None,
                    });
                }

                let connector = if is_last { end_guide } else { fork_guide };
                sink.push(Segment {
                    text: Cow::Borrowed(connector),
                    style: Some(g_style.clone()),
                    control: None,
                });
            }

            // Render node label
            let mut label_segs = Vec::new();
            self.label.render_to(console, options, &mut label_segs);

            // Handle multi-line labels
            let lines = Segment::split_lines(label_segs);
            let num_lines = lines.len();

            for (line_idx, line) in lines.into_iter().enumerate() {
                if line_idx > 0 && !is_root {
                    // Continuation prefix
                    for &(ancestor_last, ref ancestor_style) in prefixes {
                        let text = if ancestor_last {
                            space_guide
                        } else {
                            cont_guide
                        };
                        sink.push(Segment {
                            text: Cow::Borrowed(text),
                            style: Some(ancestor_style.clone()),
                            control: None,
                        });
                    }
                    let continuation = if is_last { space_guide } else { cont_guide };
                    sink.push(Segment {
                        text: Cow::Borrowed(continuation),
                        style: Some(g_style.clone()),
                        control: None,
                    });
                }

                for seg in line {
                    sink.push(seg);
                }
                sink.push(Segment::line());
            }

            if num_lines == 0 {
                sink.push(Segment::line());
            }
        }

        // Render children
        if self.expanded && !self.children.is_empty() {
            let mut next_prefixes = prefixes.to_vec();
            if !is_root {
                next_prefixes.push((is_last, g_style));
            }

            let num_children = self.children.len();
            for (idx, child) in self.children.iter().enumerate() {
                let child_is_last = idx + 1 == num_children;
                child.render_node(console, options, &next_prefixes, child_is_last, false, sink);
            }
        }
    }
}

impl Renderable for Tree {
    fn render_to<'a>(
        &'a self,
        console: &Console,
        options: &ConsoleOptions,
        sink: &mut Vec<Segment<'a>>,
    ) {
        self.render_node(console, options, &[], true, true, sink);
    }

    fn measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
        let label_m = self.label.measure(console, options);
        let mut max_w = label_m.maximum;
        for child in &self.children {
            let child_m = child.measure(console, options);
            max_w = max_w.max(child_m.maximum + 4);
        }
        Measurement::new(label_m.minimum, max_w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::ColorSystem;

    #[test]
    fn test_tree_rendering() {
        let console = Console::with_dimensions(30, 10).with_color_system(ColorSystem::NoColor);
        let mut tree = Tree::new("Root");
        tree.add("Child 1");
        let sub = tree.add("Child 2");
        sub.add("Subchild 2.1");
        tree.add("Child 3");

        let out = console.capture(|c| {
            c.print(tree);
        });

        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines[0], "Root");
        assert_eq!(lines[1], "├── Child 1");
        assert_eq!(lines[2], "├── Child 2");
        assert_eq!(lines[3], "│   └── Subchild 2.1");
        assert_eq!(lines[4], "└── Child 3");
    }

    #[test]
    fn test_tree_ascii() {
        let console = Console::with_dimensions(30, 10).with_color_system(ColorSystem::NoColor);
        let mut tree = Tree::new("Root").guides(ASCII_GUIDES);
        tree.add("A");
        tree.add("B");

        let out = console.capture(|c| {
            c.print(tree);
        });

        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines[0], "Root");
        assert_eq!(lines[1], "+-- A");
        assert_eq!(lines[2], "`-- B");
    }
}
