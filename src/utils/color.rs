pub fn hsb_to_hsv(hue: f32, saturation: f32, brightness: f32) -> (f32, f32, f32) {
    (
        hue / 65535.0 * 360.0,
        saturation / 255.0 * 100.0,
        brightness / 255.0 * 100.0,
    )
}

pub fn hsv_to_hsb(hue: f32, saturation: f32, brightness: f32) -> (f32, f32, f32) {
    (
        hue / 360.0 * 65535.0,
        saturation / 100.0 * 255.0,
        brightness / 100.0 * 255.0,
    )
}

pub fn rgb_to_hsv(red: u8, green: u8, blue: u8) -> (f32, f32, f32) {
    let red = red as f32 / 255.0;
    let green = green as f32 / 255.0;
    let blue = blue as f32 / 255.0;

    let mut hsv = (0.0, 0.0, 0.0);

    let max = red.max(green.max(blue));
    let min = red.min(green.min(blue));

    hsv.2 = max;

    let delta = max - min;

    if delta != 0.0 {
        hsv.1 = delta / max;

        if max == red {
            hsv.0 = (green - blue) / delta;
        } else if max == green {
            hsv.0 = 2.0 + (blue - red) / delta;
        } else {
            hsv.0 = 4.0 + (red - green) / delta;
        }

        hsv.0 *= 60.0;

        if hsv.0 < 0.0 {
            hsv.0 += 360.0;
        }
    }

    hsv
}

pub fn hsv_to_rgb(hue: f32, saturation: f32, brightness: f32) -> (u8, u8, u8) {
    let mut rgb = (0, 0, 0);

    let mut h = hue / 60.0;
    let s = saturation / 100.0;
    let v = brightness / 100.0;

    let i = h.floor() as i32;
    let f = h - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));

    match i {
        0 => {
            rgb.0 = (v * 255.0) as u8;
            rgb.1 = (t * 255.0) as u8;
            rgb.2 = (p * 255.0) as u8;
        }
        1 => {
            rgb.0 = (q * 255.0) as u8;
            rgb.1 = (v * 255.0) as u8;
            rgb.2 = (p * 255.0) as u8;
        }
        2 => {
            rgb.0 = (p * 255.0) as u8;
            rgb.1 = (v * 255.0) as u8;
            rgb.2 = (t * 255.0) as u8;
        }
        3 => {
            rgb.0 = (p * 255.0) as u8;
            rgb.1 = (q * 255.0) as u8;
            rgb.2 = (v * 255.0) as u8;
        }
        4 => {
            rgb.0 = (t * 255.0) as u8;
            rgb.1 = (p * 255.0) as u8;
            rgb.2 = (v * 255.0) as u8;
        }
        _ => {
            rgb.0 = (v * 255.0) as u8;
            rgb.1 = (p * 255.0) as u8;
            rgb.2 = (q * 255.0) as u8;
        }
    }

    rgb
}
