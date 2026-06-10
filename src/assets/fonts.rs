use iced::Font;

pub const RAJDHANI_REGULAR: &[u8] = include_bytes!("fonts/rajdhani-regular.ttf");
pub const RAJDHANI_MEDIUM: &[u8] = include_bytes!("fonts/rajdhani-medium.ttf");
pub const RAJDHANI_SEMIBOLD: &[u8] = include_bytes!("fonts/rajdhani-semibold.ttf");
pub const RAJDHANI_BOLD: &[u8] = include_bytes!("fonts/rajdhani-bold.ttf");
pub const IBM_PLEX_MONO_REGULAR: &[u8] = include_bytes!("fonts/ibm-plex-mono-regular.ttf");

pub const UI_FAMILY: &str = "Rajdhani";
pub const MONO_FAMILY: &str = "IBM Plex Mono";

pub const UI: Font = Font::with_name(UI_FAMILY);
pub const MONO: Font = Font::with_name(MONO_FAMILY);

pub const FONT_BYTES: &[&[u8]] = &[
    RAJDHANI_REGULAR,
    RAJDHANI_MEDIUM,
    RAJDHANI_SEMIBOLD,
    RAJDHANI_BOLD,
    IBM_PLEX_MONO_REGULAR,
];
