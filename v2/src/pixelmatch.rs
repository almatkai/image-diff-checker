//! Translation of pixelmatch 7.1.0 (ISC, Mapbox/Vladimir Agafonkin).
//! Source uses only threshold; all other options are the upstream defaults.
//! Keep arithmetic order, traversal, alpha background, and Uint8 truncation.

fn equal(a: &[u8], b: &[u8], k: usize, m: usize) -> bool {
    a[k..k + 4] == b[m..m + 4]
}
fn delta(a: &[u8], b: &[u8], k: usize, m: usize, y_only: bool) -> f64 {
    let (r1, g1, b1, a1) = (
        a[k] as f64,
        a[k + 1] as f64,
        a[k + 2] as f64,
        a[k + 3] as f64,
    );
    let (r2, g2, b2, a2) = (
        b[m] as f64,
        b[m + 1] as f64,
        b[m + 2] as f64,
        b[m + 3] as f64,
    );
    let (mut dr, mut dg, mut db, da) = (r1 - r2, g1 - g2, b1 - b2, a1 - a2);
    if dr == 0.0 && dg == 0.0 && db == 0.0 && da == 0.0 {
        return 0.0;
    }
    if a1 < 255.0 || a2 < 255.0 {
        let rb = 48.0 + 159.0 * (k % 2) as f64;
        let gb = 48.0 + 159.0 * (((k as f64 / 1.618033988749895) as i32) % 2) as f64;
        let bb = 48.0 + 159.0 * (((k as f64 / 2.618033988749895) as i32) % 2) as f64;
        dr = (r1 * a1 - r2 * a2 - rb * da) / 255.0;
        dg = (g1 * a1 - g2 * a2 - gb * da) / 255.0;
        db = (b1 * a1 - b2 * a2 - bb * da) / 255.0;
    }
    let y = dr * 0.29889531 + dg * 0.58662247 + db * 0.11448223;
    if y_only {
        return y;
    }
    let i = dr * 0.59597799 - dg * 0.27417610 - db * 0.32180189;
    let q = dr * 0.21147017 - dg * 0.52261711 + db * 0.31114694;
    let d = 0.5053 * y * y + 0.299 * i * i + 0.1957 * q * q;
    if y > 0.0 {
        -d
    } else {
        d
    }
}
fn siblings(a: &[u8], x: usize, y: usize, w: usize, h: usize) -> bool {
    let (x0, y0, x2, y2) = (
        x.saturating_sub(1),
        y.saturating_sub(1),
        (x + 1).min(w - 1),
        (y + 1).min(h - 1),
    );
    let mut zeroes = usize::from(x == x0 || x == x2 || y == y0 || y == y2);
    for xx in x0..=x2 {
        for yy in y0..=y2 {
            if xx == x && yy == y {
                continue;
            }
            zeroes += usize::from(equal(a, a, (y * w + x) * 4, (yy * w + xx) * 4));
            if zeroes > 2 {
                return true;
            }
        }
    }
    false
}
fn antialiased(a: &[u8], b: &[u8], x: usize, y: usize, w: usize, h: usize) -> bool {
    let (x0, y0, x2, y2) = (
        x.saturating_sub(1),
        y.saturating_sub(1),
        (x + 1).min(w - 1),
        (y + 1).min(h - 1),
    );
    let mut zeroes = usize::from(x == x0 || x == x2 || y == y0 || y == y2);
    let (mut min, mut max) = (0.0, 0.0);
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (0, 0, 0, 0);
    for xx in x0..=x2 {
        for yy in y0..=y2 {
            if xx == x && yy == y {
                continue;
            }
            let d = delta(a, a, (y * w + x) * 4, (yy * w + xx) * 4, true);
            if d == 0.0 {
                zeroes += 1;
                if zeroes > 2 {
                    return false;
                }
            } else if d < min {
                min = d;
                min_x = xx;
                min_y = yy;
            } else if d > max {
                max = d;
                max_x = xx;
                max_y = yy;
            }
        }
    }
    if min == 0.0 || max == 0.0 {
        return false;
    }
    (siblings(a, min_x, min_y, w, h) && siblings(b, min_x, min_y, w, h))
        || (siblings(a, max_x, max_y, w, h) && siblings(b, max_x, max_y, w, h))
}
fn gray(a: &[u8], k: usize) -> [u8; 4] {
    let v = (255.0
        + (a[k] as f64 * 0.29889531 + a[k + 1] as f64 * 0.58662247 + a[k + 2] as f64 * 0.11448223
            - 255.0)
            * 0.1
            * a[k + 3] as f64
            / 255.0) as u8;
    [v, v, v, 255]
}
pub fn compare(
    a: &[u8],
    b: &[u8],
    w: u32,
    h: u32,
    threshold: f64,
) -> Result<(Vec<u8>, usize), String> {
    let (w, h) = (w as usize, h as usize);
    if w == 0
        || h == 0
        || a.len() != w * h * 4
        || b.len() != a.len()
        || !threshold.is_finite()
        || !(0.0..=1.0).contains(&threshold)
    {
        return Err("Invalid pixelmatch input".into());
    }
    let mut out = vec![0; a.len()];
    let identical = a == b;
    let max_delta = 35215.0 * threshold * threshold;
    let mut count = 0;
    for y in 0..h {
        for x in 0..w {
            let k = (y * w + x) * 4;
            let d = if identical || equal(a, b, k, k) {
                0.0
            } else {
                delta(a, b, k, k, false)
            };
            let pixel = if d.abs() > max_delta {
                if antialiased(a, b, x, y, w, h) || antialiased(b, a, x, y, w, h) {
                    [255, 255, 0, 255]
                } else {
                    count += 1;
                    [255, 0, 0, 255]
                }
            } else {
                gray(a, k)
            };
            out[k..k + 4].copy_from_slice(&pixel);
        }
    }
    Ok((out, count))
}
