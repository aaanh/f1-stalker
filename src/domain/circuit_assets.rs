use image::ImageFormat;
use openf1::Meeting;

pub fn circuit_image_url(meeting: &Meeting) -> Option<&str> {
    let url = meeting.circuit_image.trim();
    if url.is_empty() || circuit_image_unavailable(url) {
        None
    } else {
        Some(url)
    }
}

pub fn circuit_image_unavailable(url: &str) -> bool {
    url.contains("Barcelona-Catalunya")
}

pub fn is_circuit_image_url(url: &str) -> bool {
    url.contains("carbon") || url.contains("Track%20icons") || url.contains("Track icons")
}

pub fn prepare_circuit_image(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let image = image::load_from_memory(bytes).map_err(|error| error.to_string())?;
    let mut rgba = image.to_rgba8();

    for pixel in rgba.pixels_mut() {
        if pixel[3] == 0 {
            continue;
        }
        pixel[0] = 255 - pixel[0];
        pixel[1] = 255 - pixel[1];
        pixel[2] = 255 - pixel[2];
    }

    let mut out = Vec::new();
    image::DynamicImage::ImageRgba8(rgba)
        .write_to(&mut std::io::Cursor::new(&mut out), ImageFormat::Png)
        .map_err(|error| error.to_string())?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn prepare_circuit_image_inverts_opaque_pixels() {
        let mut source = image::RgbaImage::from_pixel(2, 1, Rgba([10, 20, 30, 255]));
        source.put_pixel(1, 0, Rgba([0, 0, 0, 0]));

        let mut png = Vec::new();
        image::DynamicImage::ImageRgba8(source)
            .write_to(&mut std::io::Cursor::new(&mut png), ImageFormat::Png)
            .expect("encode png");

        let prepared = prepare_circuit_image(&png).expect("prepare circuit image");
        let prepared = image::load_from_memory(&prepared)
            .expect("decode prepared png")
            .to_rgba8();

        assert_eq!(prepared.dimensions(), (2, 1));
        assert_eq!(prepared.get_pixel(0, 0), &Rgba([245, 235, 225, 255]));
        assert_eq!(prepared.get_pixel(1, 0), &Rgba([0, 0, 0, 0]));
    }

    #[test]
    fn barcelona_catalunya_circuit_image_is_unavailable() {
        let url = "https://media.formula1.com/content/dam/fom-website/2018-redesign-assets/Track%20icons%204x3/Barcelona-Catalunya%20carbon.png";
        assert!(circuit_image_unavailable(url));
        assert!(circuit_image_url(&Meeting {
            circuit_image: url.into(),
            circuit_info_url: String::new(),
            circuit_key: 1,
            circuit_short_name: "Catalunya".into(),
            circuit_type: "Permanent".into(),
            country_code: "ESP".into(),
            country_flag: String::new(),
            country_key: 1,
            country_name: "Spain".into(),
            date_end: String::new(),
            date_start: String::new(),
            gmt_offset: String::new(),
            is_cancelled: false,
            location: "Barcelona".into(),
            meeting_key: 1,
            meeting_name: "Barcelona Grand Prix".into(),
            meeting_official_name: String::new(),
            year: 2026,
        })
        .is_none());
    }
}
