use iced::Color;
use openf1::Driver;

use crate::db::PinnedDriver;
use crate::ui::theme::muted;

#[derive(Debug, Clone)]
pub struct PinnedDriverView {
    pub driver: Driver,
}

pub fn driver_display_name(driver: &Driver) -> &str {
    if driver.name_acronym.is_empty() {
        &driver.full_name
    } else {
        &driver.name_acronym
    }
}

pub fn team_colour(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return muted();
    }

    let parse = |start: usize| u8::from_str_radix(&hex[start..start + 2], 16).unwrap_or(128);

    Color::from_rgb8(parse(0), parse(2), parse(4))
}

pub fn pinned_driver_views(pins: &[PinnedDriver], roster: &[Driver]) -> Vec<PinnedDriverView> {
    pins.iter()
        .filter_map(|pin| {
            roster
                .iter()
                .find(|driver| driver.driver_number == pin.driver_number)
                .cloned()
                .map(|driver| PinnedDriverView { driver })
        })
        .collect()
}

pub fn can_pin(pins: &[PinnedDriver], driver_number: i64) -> bool {
    !pins
        .iter()
        .any(|pin| pin.driver_number == driver_number)
}

pub fn pin_driver(pins: &mut Vec<PinnedDriver>, driver_number: i64) -> bool {
    if !can_pin(pins, driver_number) {
        return false;
    }

    pins.push(PinnedDriver {
        driver_number,
        sort_order: pins.len() as i32,
    });
    reindex_pins(pins);
    true
}

pub fn unpin_driver(pins: &mut Vec<PinnedDriver>, driver_number: i64) -> bool {
    let before = pins.len();
    pins.retain(|pin| pin.driver_number != driver_number);
    if pins.len() == before {
        return false;
    }
    reindex_pins(pins);
    true
}

pub fn unpin_all(pins: &mut Vec<PinnedDriver>) -> bool {
    if pins.is_empty() {
        return false;
    }
    pins.clear();
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinDirection {
    Left,
    Right,
}

pub fn move_pin(pins: &mut Vec<PinnedDriver>, driver_number: i64, direction: PinDirection) -> bool {
    let Some(index) = pins.iter().position(|pin| pin.driver_number == driver_number) else {
        return false;
    };

    let target = match direction {
        PinDirection::Left if index > 0 => index - 1,
        PinDirection::Right if index + 1 < pins.len() => index + 1,
        _ => return false,
    };

    pins.swap(index, target);
    reindex_pins(pins);
    true
}

pub fn reindex_pins(pins: &mut [PinnedDriver]) {
    for (index, pin) in pins.iter_mut().enumerate() {
        pin.sort_order = index as i32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_driver(number: i64) -> Driver {
        Driver {
            broadcast_name: format!("D{number}"),
            country_code: "GB".into(),
            driver_number: number,
            first_name: "Test".into(),
            full_name: format!("Driver {number}"),
            headshot_url: String::new(),
            last_name: "Driver".into(),
            meeting_key: 1,
            name_acronym: format!("D{number}"),
            session_key: 1,
            team_colour: "FF0000".into(),
            team_name: "Team".into(),
        }
    }

    #[test]
    fn pins_reject_duplicates() {
        let mut pins = Vec::new();
        assert!(pin_driver(&mut pins, 44));
        assert!(!pin_driver(&mut pins, 44));
        assert_eq!(pins.len(), 1);
    }

    #[test]
    fn pins_allow_unlimited_drivers() {
        let mut pins = Vec::new();
        for number in 1..=20 {
            assert!(pin_driver(&mut pins, number as i64));
        }
        assert_eq!(pins.len(), 20);
    }

    #[test]
    fn move_pin_swaps_neighbors() {
        let mut pins = vec![
            PinnedDriver {
                driver_number: 44,
                sort_order: 0,
            },
            PinnedDriver {
                driver_number: 16,
                sort_order: 1,
            },
        ];
        assert!(move_pin(&mut pins, 16, PinDirection::Left));
        assert_eq!(pins[0].driver_number, 16);
        assert_eq!(pins[1].driver_number, 44);
    }

    #[test]
    fn pinned_views_follow_sort_order() {
        let pins = vec![
            PinnedDriver {
                driver_number: 16,
                sort_order: 0,
            },
            PinnedDriver {
                driver_number: 44,
                sort_order: 1,
            },
        ];
        let roster = vec![sample_driver(44), sample_driver(16)];
        let views = pinned_driver_views(&pins, &roster);
        assert_eq!(views.len(), 2);
        assert_eq!(views[0].driver.driver_number, 16);
        assert_eq!(views[1].driver.driver_number, 44);
    }
}
