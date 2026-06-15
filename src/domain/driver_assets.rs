pub fn driver_flag_url(
    country_code: &str,
    driver_number: i64,
    name_acronym: &str,
) -> Option<String> {
    driver_flag_iso2(country_code, driver_number, name_acronym)
        .as_deref()
        .map(flag_cdn_url)
}

pub fn driver_flag_iso2(
    country_code: &str,
    driver_number: i64,
    name_acronym: &str,
) -> Option<String> {
    if let Some(iso2) = country_code_to_iso2(country_code) {
        return Some(iso2);
    }

    if let Some(iso2) = driver_acronym_to_iso2(name_acronym) {
        return Some(iso2.to_string());
    }

    driver_number_to_iso2(driver_number).map(str::to_string)
}

pub fn team_logo_url(team_name: &str) -> Option<String> {
    team_logo_cdn_url(team_name, 256, 256)
}

pub fn team_logo_display_url(team_name: &str) -> Option<String> {
    team_logo_cdn_url(team_name, 480, 192)
}

fn team_logo_cdn_url(team_name: &str, width: u32, height: u32) -> Option<String> {
    let slug = team_logo_slug(team_name)?;
    Some(format!(
        "https://media.formula1.com/image/upload/c_fit,w_{width},h_{height}/f_auto/q_auto/content/dam/fom-website/2018-redesign-assets/team%20logos/{slug}"
    ))
}

fn flag_cdn_url(iso2: &str) -> String {
    format!(
        "https://flagcdn.com/h24/{}.png",
        iso2.trim().to_ascii_lowercase()
    )
}

fn country_code_to_iso2(country_code: &str) -> Option<String> {
    let code = country_code.trim();
    if code.is_empty() {
        return None;
    }

    if code.len() == 2 && code.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return Some(code.to_ascii_lowercase());
    }

    f1_country_code_to_iso2(code).map(str::to_string)
}

fn f1_country_code_to_iso2(country_code: &str) -> Option<&'static str> {
    match country_code.trim().to_ascii_uppercase().as_str() {
        "NED" | "NLD" | "NET" => Some("nl"),
        "GBR" | "ENG" | "UK" => Some("gb"),
        "MON" => Some("mc"),
        "GER" | "DEU" => Some("de"),
        "ESP" => Some("es"),
        "ITA" => Some("it"),
        "AUS" => Some("au"),
        "CAN" => Some("ca"),
        "JPN" => Some("jp"),
        "CHN" => Some("cn"),
        "USA" => Some("us"),
        "MEX" => Some("mx"),
        "BRA" => Some("br"),
        "AUT" => Some("at"),
        "CHE" => Some("ch"),
        "ARG" => Some("ar"),
        "DEN" => Some("dk"),
        "FIN" => Some("fi"),
        "FRA" => Some("fr"),
        "THA" => Some("th"),
        "POL" => Some("pl"),
        "NZL" => Some("nz"),
        "COL" => Some("co"),
        "POR" => Some("pt"),
        "ISR" => Some("il"),
        "UAE" => Some("ae"),
        "SAU" => Some("sa"),
        "HUN" => Some("hu"),
        "BEL" => Some("be"),
        "RSA" | "ZAF" => Some("za"),
        "INA" | "IDN" => Some("id"),
        "MAS" | "MYS" => Some("my"),
        "TPE" => Some("tw"),
        "VEN" => Some("ve"),
        "LBN" => Some("lb"),
        "LUX" => Some("lu"),
        "SMR" => Some("sm"),
        "AND" => Some("ad"),
        "LIE" => Some("li"),
        _ => None,
    }
}

/// Stable fallback when OpenF1 omits `country_code` and car numbers change.
fn driver_acronym_to_iso2(name_acronym: &str) -> Option<&'static str> {
    match name_acronym.trim().to_ascii_uppercase().as_str() {
        "VER" => Some("nl"),
        "NOR" | "HAM" | "RUS" | "BEA" | "LIN" => Some("gb"),
        "BOR" => Some("br"),
        "HAD" | "GAS" | "OCO" => Some("fr"),
        "PER" => Some("mx"),
        "ANT" => Some("it"),
        "ALO" | "SAI" => Some("es"),
        "LEC" => Some("mc"),
        "STR" => Some("ca"),
        "ALB" => Some("th"),
        "HUL" => Some("de"),
        "LAW" => Some("nz"),
        "COL" => Some("ar"),
        "BOT" => Some("fi"),
        "PIA" => Some("au"),
        "TSU" => Some("jp"),
        "DOO" => Some("au"),
        _ => None,
    }
}

/// Season-specific fallback when acronym lookup is unavailable.
fn driver_number_to_iso2(driver_number: i64) -> Option<&'static str> {
    match driver_number {
        1 => Some("gb"),   // Norris
        3 => Some("nl"),   // Verstappen
        5 => Some("br"),   // Bortoleto
        6 => Some("fr"),   // Hadjar
        10 => Some("fr"),  // Gasly
        11 => Some("mx"),  // Perez
        12 => Some("it"),  // Antonelli
        14 => Some("es"),  // Alonso
        16 => Some("mc"),  // Leclerc
        18 => Some("ca"),  // Stroll
        23 => Some("th"),  // Albon
        27 => Some("de"),  // Hulkenberg
        30 => Some("nz"),  // Lawson
        31 => Some("fr"),  // Ocon
        41 => Some("gb"),  // Lindblad
        43 => Some("ar"),  // Colapinto
        44 => Some("gb"),  // Hamilton
        55 => Some("es"),  // Sainz
        63 => Some("gb"),  // Russell
        77 => Some("fi"),  // Bottas
        81 => Some("au"),  // Piastri
        87 => Some("gb"),  // Bearman
        _ => None,
    }
}

fn team_logo_slug(team_name: &str) -> Option<&'static str> {
    let lower = team_name.to_lowercase();

    if lower.contains("racing bulls") || lower.contains("visa cash app rb") {
        return Some("rb");
    }
    if lower.contains("red bull") {
        return Some("red%20bull");
    }
    if lower.contains("mclaren") {
        return Some("mclaren");
    }
    if lower.contains("ferrari") {
        return Some("ferrari");
    }
    if lower.contains("mercedes") {
        return Some("mercedes");
    }
    if lower.contains("aston martin") {
        return Some("aston%20martin");
    }
    if lower.contains("alpine") {
        return Some("alpine");
    }
    if lower.contains("williams") {
        return Some("williams");
    }
    if lower.contains("haas") {
        return Some("haas");
    }
    if lower.contains("kick sauber")
        || lower.contains("stake f1")
        || lower.contains("alfa romeo")
        || lower.contains("sauber")
    {
        return Some("kick%20sauber");
    }
    if lower.contains("alphatauri") {
        return Some("alphatauri");
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_common_f1_country_codes() {
        assert_eq!(
            driver_flag_url("NED", 0, "").as_deref(),
            Some("https://flagcdn.com/h24/nl.png")
        );
        assert_eq!(
            driver_flag_url("GBR", 0, "").as_deref(),
            Some("https://flagcdn.com/h24/gb.png")
        );
        assert_eq!(
            driver_flag_url("gb", 0, "").as_deref(),
            Some("https://flagcdn.com/h24/gb.png")
        );
    }

    #[test]
    fn falls_back_to_acronym_when_country_code_missing() {
        assert_eq!(
            driver_flag_url("", 3, "VER").as_deref(),
            Some("https://flagcdn.com/h24/nl.png")
        );
        assert_eq!(
            driver_flag_iso2("", 1, "NOR").as_deref(),
            Some("gb")
        );
    }

    #[test]
    fn falls_back_to_driver_number_when_country_code_missing() {
        assert_eq!(
            driver_flag_url("", 44, "").as_deref(),
            Some("https://flagcdn.com/h24/gb.png")
        );
        assert_eq!(
            driver_flag_iso2("", 81, "").as_deref(),
            Some("au")
        );
    }

    #[test]
    fn maps_constructor_logos() {
        assert!(team_logo_url("Red Bull Racing").is_some());
        assert!(team_logo_url("Racing Bulls").is_some());
        assert!(team_logo_url("McLaren").is_some());
        assert!(team_logo_url("Unknown GP Team").is_none());
    }

    #[test]
    fn distinguishes_red_bull_from_racing_bulls() {
        let bulls = team_logo_url("Red Bull Racing").unwrap();
        let rb = team_logo_url("Racing Bulls").unwrap();
        assert!(bulls.contains("red%20bull"));
        assert!(bulls.contains("c_fit"));
        assert!(!bulls.contains("c_lfill"));
        assert!(rb.contains("/rb"));
    }

    #[test]
    fn display_logo_uses_wide_fit_bounds() {
        let url = team_logo_display_url("Ferrari").unwrap();
        assert!(url.contains("c_fit"));
        assert!(url.contains("w_480"));
        assert!(url.contains("h_192"));
    }
}
