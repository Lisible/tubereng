/// Returns the perceived lightness for a color in ``sRGB`` color space
///
/// The returned value is in the 0.0-1.0 range
#[must_use]
pub(crate) fn srgb_perceived_lightness(r: f32, g: f32, b: f32) -> f32 {
    let luminance = srgb_luminance(r, g, b);
    let perceived_lightness = if luminance <= 216.0 / 24389.0 {
        luminance * (24389.0 / 27.0)
    } else {
        luminance.powf(1.0 / 3.0) * 116.0 - 16.0
    };
    perceived_lightness / 100.0
}

fn srgb_luminance(r: f32, g: f32, b: f32) -> f32 {
    let r_linear = srgb_to_linear(r);
    let g_linear = srgb_to_linear(g);
    let b_linear = srgb_to_linear(b);
    r_linear * 0.2126 + g_linear * 0.7152 + b_linear * 0.0722
}

fn srgb_to_linear(color_channel: f32) -> f32 {
    if color_channel <= 0.04045 {
        color_channel / 12.92
    } else {
        ((color_channel + 0.055) / 1.055).powf(2.4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perceived_lightness() {
        assert!(srgb_perceived_lightness(1.0, 1.0, 1.0) > 0.9);
        assert!(srgb_perceived_lightness(1.0, 1.0, 1.0) < 1.1);
        assert!(srgb_perceived_lightness(0.5, 0.5, 0.5) > 0.52);
        assert!(srgb_perceived_lightness(0.5, 0.5, 0.5) < 0.54);
    }
}
