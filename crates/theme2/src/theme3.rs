use gpui::{hsla, rgba, HighlightStyle, Hsla, SharedString};

/// Turn a hex string into an Hsla color
/// Accepts 3, 4, 6, or 8 digit hex strings
///
/// # Examples
/// ```
/// use gpui::Hsla;
/// use theme3::hex;
///
/// assert_eq!(hex("000"), Hsla::new(0.0, 0.0, 0.0, 1.0));
/// assert_eq!(hex("0000"), Hsla::new(0.0, 0.0, 0.0, 0.0));
/// assert_eq!(hex("000000"), Hsla::new(0.0, 0.0, 0.0, 1.0));
/// assert_eq!(hex("00000000"), Hsla::new(0.0, 0.0, 0.0, 0.0));
/// ```
pub fn hex(s: &str) -> Hsla {
    let mut hex = s.to_string();

    if hex.starts_with('#') {
        hex = hex[1..].to_string();
    }

    if hex.len() == 3 {
        let mut new_hex = String::with_capacity(6);
        for c in hex.chars() {
            new_hex.push(c);
            new_hex.push(c);
        }
        hex = hex;
    }

    if hex.len() == 4 {
        let mut new_hex = String::with_capacity(8);
        for c in hex.chars() {
            new_hex.push(c);
            new_hex.push(c);
        }
        hex = hex;
    }

    if hex.len() == 6 {
        hex = format!("{}{}", s, "ff");
    }

    if s.len() == 8 {
        hex = s.to_string();
    }

    let hex = u32::from_str_radix(&hex, 16).unwrap();

    rgba(hex).into()
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Appearance {
    Light,
    Dark,
}

impl From<String> for Appearance {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "light" => Self::Light,
            "dark" => Self::Dark,
            _ => Self::Light,
        }
    }
}

impl Appearance {
    pub fn is_light(&self) -> bool {
        match self {
            Self::Light => true,
            Self::Dark => false,
        }
    }
}

pub struct ThemeFamily {
    pub id: String,
    pub name: SharedString,
    pub author: SharedString,
    pub themes: Vec<Theme>,
}

impl ThemeFamily {}

pub struct Theme {
    pub id: String,
    pub name: SharedString,
    pub appearance: Appearance,
    pub styles: ThemeStyles,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PlayerColor {
    pub cursor: Hsla,
    pub background: Hsla,
    pub selection: Hsla,
}

impl From<Hsla> for PlayerColor {
    fn from(color: Hsla) -> Self {
        Self {
            cursor: color,
            background: ColorTools::alpha_color(color, 0.2),
            selection: ColorTools::alpha_color(color, 0.12),
        }
    }
}

pub struct ColorTools {}

impl ColorTools {
    pub fn alpha_color(color: Hsla, alpha: f32) -> Hsla {
        hsla(color.h, color.s, color.l, alpha)
    }
}

#[derive(Clone, Debug)]
pub struct ThemeStyles {
    pub background: Hsla,
    pub background_selected: Hsla,
    pub background_variant: Hsla,
    pub border: Hsla,
    pub border_selected: Hsla,
    pub border_variant: Hsla,
    pub button_background: Hsla,
    pub drag_target_background: Hsla,
    pub editor_active_line_background: Hsla,
    pub editor_active_line_number: Hsla,
    pub editor_active_wrap_guide: Hsla,
    pub editor_background: Hsla,
    pub editor_document_highlight_read_background: Hsla,
    pub editor_document_highlight_write_background: Hsla,
    pub editor_foreground: Hsla,
    pub editor_gutter_background: Hsla,
    pub editor_highlighted_line_background: Hsla,
    pub editor_invisible: Hsla,
    pub editor_line_number: Hsla,
    pub editor_occurrence: Hsla,
    pub editor_predictive: Hsla,
    pub editor_subheader_background: Hsla,
    pub editor_unreachable: Hsla,
    pub editor_wrap_guide: Hsla,
    pub foreground: Hsla,
    pub foreground_accent: Hsla,
    pub foreground_placeholder: Hsla,
    pub foreground_selected: Hsla,
    pub foreground_variant: Hsla,
    pub highlights: Vec<(String, HighlightStyle)>,
    pub input_background: Hsla,
    pub input_border: Hsla,
    pub modal_background: Hsla,
    pub pane_background: Hsla,
    pub pane_focused_border: Hsla,
    pub panel_background: Hsla,
    pub panel_button: Hsla,
    pub panel_focused_border: Hsla,
    pub player_host: PlayerColor,
    pub players: Vec<PlayerColor>,
    pub popover_background: Hsla,
    pub popover_border: Hsla,
    pub scrollbar_thumb: Hsla,
    pub scrollbar_thumb_border: Hsla,
    pub scrollbar_track_background: Hsla,
    pub scrollbar_track_border: Hsla,
    pub search_match: Hsla,
    pub shadow: PlayerColor,
    pub state_focused: Hsla,
    pub state_selected: Hsla,
    pub status_bar_background: Hsla,
    pub status_conflict: Hsla,
    pub status_created: Hsla,
    pub status_deleted: Hsla,
    pub status_error: Hsla,
    pub status_hidden: Hsla,
    pub status_hint: Hsla,
    pub status_ignored: Hsla,
    pub status_info: Hsla,
    pub status_modified: Hsla,
    pub status_renamed: Hsla,
    pub status_success: Hsla,
    pub status_warning: Hsla,
    pub tab_active_background: Hsla,
    pub tab_bar_background: Hsla,
    pub tab_inactive_background: Hsla,
    pub terminal: TerminalColors,
    pub title_bar_background: Hsla,
    pub title_bar_button: Hsla,
    pub toolbar_background: Hsla,
    pub toolbar_button: Hsla,
}

#[derive(Clone, Debug)]
pub struct TerminalColors {
    pub background: Hsla,
    pub foreground: Hsla,
    pub cursor: Hsla,
    pub ansi_bright_black: Hsla,
    pub ansi_bright_red: Hsla,
    pub ansi_bright_green: Hsla,
    pub ansi_bright_yellow: Hsla,
    pub ansi_bright_blue: Hsla,
    pub ansi_bright_magenta: Hsla,
    pub ansi_bright_cyan: Hsla,
    pub ansi_bright_white: Hsla,
    pub ansi_black: Hsla,
    pub ansi_red: Hsla,
    pub ansi_green: Hsla,
    pub ansi_yellow: Hsla,
    pub ansi_blue: Hsla,
    pub ansi_magenta: Hsla,
    pub ansi_cyan: Hsla,
    pub ansi_white: Hsla,
}
