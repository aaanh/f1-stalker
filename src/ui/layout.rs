use iced::{Length, Size};

#[derive(Debug, Clone, Copy)]
pub struct LayoutConfig {
    pub viewport: Size,
    pub padding: f32,
    pub countdown_size: u16,
    pub meeting_title_size: u16,
    pub card_heading_size: u16,
    pub card_body_size: u16,
    pub card_detail_size: u16,
    pub card_padding: f32,
    pub card_flag_width: f32,
    pub card_flag_height: f32,
    pub card_track_height: f32,
    pub stack_header: bool,
    pub stack_cards: bool,
    pub card_width: Length,
    pub pinned_grid_columns: usize,
    pub pin_card_stacked: bool,
    pub pin_portrait_size: f32,
    pub pin_name_size: u16,
}

impl LayoutConfig {
    pub fn from_viewport(viewport: Size) -> Self {
        let width = viewport.width;
        let height = viewport.height;

        let stack_cards = width < 920.0;
        let stack_header = width < 720.0;
        let compact = width < 920.0 || height < 720.0;
        let card_compact = width < 820.0 || height < 680.0;
        let tight = height < 680.0 || width < 820.0;
        let pinned_grid_columns = pinned_grid_columns_for_width(width);
        let padding = if tight {
            12.0
        } else if compact {
            16.0
        } else {
            24.0
        };
        let pin_slot_width =
            pinned_slot_width(width, padding, pinned_grid_columns);
        let pin_card_stacked =
            pin_card_stacked_for_slot(pin_slot_width, pinned_grid_columns);
        let pin_portrait_size = if pin_card_stacked {
            88.0
        } else if card_compact {
            96.0
        } else {
            112.0
        };
        let pin_name_size = if pin_card_stacked {
            16
        } else if card_compact {
            17
        } else {
            18
        };

        Self {
            viewport,
            padding,
            countdown_size: if width < 720.0 {
                28
            } else if compact {
                34
            } else {
                46
            },
            meeting_title_size: if card_compact { 19 } else { 22 },
            card_heading_size: if card_compact { 13 } else { 14 },
            card_body_size: if card_compact { 15 } else { 17 },
            card_detail_size: if card_compact { 13 } else { 15 },
            card_padding: if tight { 16.0 } else { 20.0 },
            card_flag_width: if card_compact { 42.0 } else { 50.0 },
            card_flag_height: if card_compact { 28.0 } else { 34.0 },
            card_track_height: if card_compact { 152.0 } else { 192.0 },
            stack_header,
            stack_cards,
            card_width: if stack_cards {
                Length::Fill
            } else {
                Length::FillPortion(1)
            },
            pinned_grid_columns,
            pin_card_stacked,
            pin_portrait_size,
            pin_name_size,
        }
    }

    /// Fixed height for side-by-side race cards. Scrollable content cannot use
    /// vertical `Fill`, so equal heights are enforced with a shared fixed size.
    pub fn card_height(self) -> Length {
        if self.stack_cards {
            Length::Shrink
        } else {
            Length::Fixed(430.0)
        }
    }
}

fn pinned_grid_columns_for_width(width: f32) -> usize {
    if width < 720.0 {
        1
    } else if width < 920.0 {
        2
    } else {
        3
    }
}

const PINNED_SECTION_PADDING: f32 = 32.0;
const PINNED_GRID_GAP: f32 = 12.0;
/// Horizontal pin cards need portrait, identity, and controls in one row.
const PIN_CARD_STACK_BREAKPOINT: f32 = 460.0;

pub fn pinned_slot_width(viewport_width: f32, dashboard_padding: f32, columns: usize) -> f32 {
    let columns = columns.max(1);
    let content_width = viewport_width - dashboard_padding * 2.0;
    let section_width = content_width - PINNED_SECTION_PADDING;
    let gap_total = PINNED_GRID_GAP * (columns - 1) as f32;
    (section_width - gap_total) / columns as f32
}

fn pin_card_stacked_for_slot(slot_width: f32, columns: usize) -> bool {
    columns == 1 || slot_width < PIN_CARD_STACK_BREAKPOINT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pin_cards_stack_in_three_column_grid_at_default_width() {
        let columns = pinned_grid_columns_for_width(1100.0);
        let slot = pinned_slot_width(1100.0, 24.0, columns);
        assert_eq!(columns, 3);
        assert!(slot < PIN_CARD_STACK_BREAKPOINT);
        assert!(pin_card_stacked_for_slot(slot, columns));
    }

    #[test]
    fn pin_cards_use_horizontal_layout_in_wide_three_column_grid() {
        let columns = pinned_grid_columns_for_width(1600.0);
        let slot = pinned_slot_width(1600.0, 24.0, columns);
        assert_eq!(columns, 3);
        assert!(slot >= PIN_CARD_STACK_BREAKPOINT);
        assert!(!pin_card_stacked_for_slot(slot, columns));
    }
}
