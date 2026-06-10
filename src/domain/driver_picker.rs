use openf1::Driver;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DriverSortField {
    #[default]
    LastName,
    FirstName,
    Code,
    Constructor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortDirection {
    #[default]
    Asc,
    Desc,
}

impl SortDirection {
    pub fn toggle(self) -> Self {
        match self {
            Self::Asc => Self::Desc,
            Self::Desc => Self::Asc,
        }
    }

    pub fn arrow(self) -> &'static str {
        match self {
            Self::Asc => "↑",
            Self::Desc => "↓",
        }
    }
}

#[derive(Debug, Clone)]
pub struct DriverPickerFilters {
    pub search: String,
    pub sort_field: DriverSortField,
    pub sort_direction: SortDirection,
    pub group_by_constructor: bool,
}

impl Default for DriverPickerFilters {
    fn default() -> Self {
        Self {
            search: String::new(),
            sort_field: DriverSortField::LastName,
            sort_direction: SortDirection::Asc,
            group_by_constructor: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DriverPickerGroup {
    pub team_name: String,
    pub drivers: Vec<Driver>,
}

pub fn filter_roster<'a>(roster: &'a [Driver], query: &str) -> Vec<&'a Driver> {
    let query = query.trim();
    if query.is_empty() {
        return roster.iter().collect();
    }

    let needle = query.to_lowercase();
    roster
        .iter()
        .filter(|driver| driver_matches(driver, &needle))
        .collect()
}

pub fn organize_roster(roster: &[Driver], filters: &DriverPickerFilters) -> Vec<DriverPickerGroup> {
    let filtered: Vec<Driver> = filter_roster(roster, &filters.search)
        .into_iter()
        .cloned()
        .collect();

    if filters.group_by_constructor {
        group_by_team(&filtered, filters)
    } else {
        let mut drivers = filtered;
        sort_drivers(&mut drivers, filters.sort_field, filters.sort_direction);
        vec![DriverPickerGroup {
            team_name: String::new(),
            drivers,
        }]
    }
}

fn group_by_team(drivers: &[Driver], filters: &DriverPickerFilters) -> Vec<DriverPickerGroup> {
    use std::collections::BTreeMap;

    let mut by_team: BTreeMap<String, Vec<Driver>> = BTreeMap::new();
    for driver in drivers {
        by_team
            .entry(driver.team_name.clone())
            .or_default()
            .push(driver.clone());
    }

    let within_field = if filters.sort_field == DriverSortField::Constructor {
        DriverSortField::LastName
    } else {
        filters.sort_field
    };

    let mut groups: Vec<DriverPickerGroup> = by_team
        .into_iter()
        .map(|(team_name, mut drivers)| {
            sort_drivers(&mut drivers, within_field, filters.sort_direction);
            DriverPickerGroup { team_name, drivers }
        })
        .collect();

    let reverse = filters.sort_direction == SortDirection::Desc;
    groups.sort_by(|left, right| {
        let ordering = left
            .team_name
            .to_lowercase()
            .cmp(&right.team_name.to_lowercase());
        if filters.sort_field == DriverSortField::Constructor && reverse {
            ordering.reverse()
        } else {
            ordering
        }
    });

    groups
}

fn sort_drivers(drivers: &mut [Driver], field: DriverSortField, direction: SortDirection) {
    drivers.sort_by(|left, right| {
        let ordering = sort_key(left, field).cmp(&sort_key(right, field));
        match direction {
            SortDirection::Asc => ordering,
            SortDirection::Desc => ordering.reverse(),
        }
    });
}

fn sort_key(driver: &Driver, field: DriverSortField) -> String {
    match field {
        DriverSortField::FirstName => driver.first_name.to_lowercase(),
        DriverSortField::LastName => driver.last_name.to_lowercase(),
        DriverSortField::Code => driver.name_acronym.to_lowercase(),
        DriverSortField::Constructor => driver.team_name.to_lowercase(),
    }
}

fn driver_matches(driver: &Driver, needle: &str) -> bool {
    [
        &driver.first_name,
        &driver.last_name,
        &driver.name_acronym,
        &driver.team_name,
        &driver.full_name,
        &driver.broadcast_name,
    ]
    .iter()
    .any(|value| value.to_lowercase().contains(needle))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_driver(number: i64, first: &str, last: &str, code: &str, team: &str) -> Driver {
        Driver {
            broadcast_name: format!("{first} {last}"),
            country_code: String::new(),
            driver_number: number,
            first_name: first.into(),
            full_name: format!("{first} {last}"),
            headshot_url: String::new(),
            last_name: last.into(),
            meeting_key: 1,
            name_acronym: code.into(),
            session_key: 1,
            team_colour: "FFFFFF".into(),
            team_name: team.into(),
        }
    }

    #[test]
    fn search_matches_acronym_and_team() {
        let roster = vec![
            sample_driver(1, "Max", "Verstappen", "VER", "Red Bull Racing"),
            sample_driver(44, "Lewis", "Hamilton", "HAM", "Ferrari"),
        ];
        let filters = DriverPickerFilters {
            search: "fer".into(),
            ..Default::default()
        };
        let groups = organize_roster(&roster, &filters);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].drivers.len(), 1);
        assert_eq!(groups[0].drivers[0].driver_number, 44);
    }

    #[test]
    fn groups_keep_team_sections() {
        let roster = vec![
            sample_driver(1, "Max", "Verstappen", "VER", "Red Bull Racing"),
            sample_driver(44, "Lewis", "Hamilton", "HAM", "Ferrari"),
            sample_driver(16, "Charles", "Leclerc", "LEC", "Ferrari"),
        ];
        let filters = DriverPickerFilters {
            group_by_constructor: true,
            sort_field: DriverSortField::LastName,
            ..Default::default()
        };
        let groups = organize_roster(&roster, &filters);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].team_name, "Ferrari");
        assert_eq!(groups[0].drivers[0].name_acronym, "HAM");
        assert_eq!(groups[1].team_name, "Red Bull Racing");
    }

    #[test]
    fn constructor_sort_desc_reorders_groups() {
        let roster = vec![
            sample_driver(1, "Max", "Verstappen", "VER", "Red Bull Racing"),
            sample_driver(44, "Lewis", "Hamilton", "HAM", "Ferrari"),
        ];
        let filters = DriverPickerFilters {
            group_by_constructor: true,
            sort_field: DriverSortField::Constructor,
            sort_direction: SortDirection::Desc,
            ..Default::default()
        };
        let groups = organize_roster(&roster, &filters);
        assert_eq!(groups[0].team_name, "Red Bull Racing");
        assert_eq!(groups[1].team_name, "Ferrari");
    }
}
