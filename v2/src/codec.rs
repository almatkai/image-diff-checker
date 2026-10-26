use base64::{engine::general_purpose::STANDARD, Engine};
use std::io::Cursor;
pub const MAX_FILE: usize = 32 * 1024 * 1024;
pub const MAX_SIDE: u32 = 8192;
pub const MAX_PIXELS: u64 = 16_000_000;
pub const MAX_PREVIEW_PNG: usize = 7_500_000;
#[derive(Debug)]
pub struct Raster {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}
pub fn dimensions(w: u32, h: u32) -> Result<(), String> {
    if w == 0 || h == 0 || w > MAX_SIDE || h > MAX_SIDE || u64::from(w) * u64::from(h) > MAX_PIXELS
    {
        return Err("Image exceeds 8192 per side or 16,000,000 decoded pixels".into());
    }
    Ok(())
}
pub fn decode(bytes: &[u8]) -> Result<Raster, String> {
    if bytes.len() > MAX_FILE {
        return Err("Selected file exceeds 32 MiB".into());
    }
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        decode_png(bytes)
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        decode_gif(bytes)
    } else if bytes.starts_with(b"\xff\xd8") {
        decode_jpeg(bytes)
    } else {
        Err("Unsupported or damaged image".into())
    }
}
fn decode_png(bytes: &[u8]) -> Result<Raster, String> {
    let mut decoder = png::Decoder::new_with_limits(
        Cursor::new(bytes),
        png::Limits {
            bytes: 160 * 1024 * 1024,
        },
    );
    decoder.set_transformations(png::Transformations::EXPAND);
    let mut reader = decoder.read_info().map_err(|e| e.to_string())?;
    let info = reader.info();
    dimensions(info.width, info.height)?;
    let trns = info.trns.is_some()
        && matches!(
            info.color_type,
            png::ColorType::Rgb | png::ColorType::Grayscale
        );
    let mut data = vec![0; reader.output_buffer_size()];
    let output = reader.next_frame(&mut data).map_err(|e| e.to_string())?;
    let data = &data[..output.buffer_size()];
    let channels = output.color_type.samples();
    let stride = if output.bit_depth == png::BitDepth::Sixteen {
        2
    } else {
        1
    };
    let mut rgba = Vec::with_capacity(output.width as usize * output.height as usize * 4);
    for px in data.chunks_exact(channels * stride) {
        let get = |n: usize| -> u8 {
            if stride == 2 {
                let v = u16::from_be_bytes([px[2 * n], px[2 * n + 1]]) as f64;
                (v * 255.0 / 65535.0 + 0.5).floor() as u8
            } else {
                px[n]
            }
        };
        let mut pixel = match output.color_type {
            png::ColorType::Rgb => [get(0), get(1), get(2), 255],
            png::ColorType::Rgba => [get(0), get(1), get(2), get(3)],
            png::ColorType::Grayscale => [get(0), get(0), get(0), 255],
            png::ColorType::GrayscaleAlpha => [get(0), get(0), get(0), get(1)],
            _ => return Err("Unexpanded PNG".into()),
        };
        // pngjs zeros all channels for tRNS matching samples, unlike RGBA/palette.
        if trns && pixel[3] == 0 {
            pixel = [0; 4];
        }
        rgba.extend_from_slice(&pixel);
    }
    Ok(Raster {
        width: output.width,
        height: output.height,
        rgba,
    })
}
fn decode_gif(bytes: &[u8]) -> Result<Raster, String> {
    let mut options = gif::DecodeOptions::new();
    options.set_color_output(gif::ColorOutput::Indexed);
    options.set_memory_limit(gif::MemoryLimit::Bytes(
        std::num::NonZeroU64::new(16_000_000).unwrap(),
    ));
    let mut reader = options
        .read_info(Cursor::new(bytes))
        .map_err(|e| e.to_string())?;
    let (w, h) = (reader.width() as u32, reader.height() as u32);
    dimensions(w, h)?;
    let palette = reader.global_palette().map(|p| p.to_vec());
    let frame = reader
        .read_next_frame()
        .map_err(|e| e.to_string())?
        .ok_or("GIF has no first frame")?;
    let palette = frame
        .palette
        .as_deref()
        .or(palette.as_deref())
        .ok_or("Missing GIF palette")?;
    if u32::from(frame.left) + u32::from(frame.width) > w
        || u32::from(frame.top) + u32::from(frame.height) > h
    {
        return Err("GIF frame outside logical canvas".into());
    }
    let mut rgba = vec![0; w as usize * h as usize * 4];
    for (n, &index) in frame.buffer.iter().enumerate() {
        if Some(index) == frame.transparent {
            continue;
        }
        let p = usize::from(index) * 3;
        let color = palette.get(p..p + 3).ok_or("Invalid GIF palette index")?;
        let x = n % usize::from(frame.width) + usize::from(frame.left);
        let y = n / usize::from(frame.width) + usize::from(frame.top);
        let k = (y * w as usize + x) * 4;
        rgba[k..k + 4].copy_from_slice(&[color[0], color[1], color[2], 255]);
    }
    Ok(Raster {
        width: w,
        height: h,
        rgba,
    })
}
#[path = "jpeg.rs"]
mod jpeg;
fn decode_jpeg(bytes: &[u8]) -> Result<Raster, String> {
    Ok(orient(jpeg::decode(bytes)?, exif_orientation(bytes)))
}
fn exif_orientation(bytes: &[u8]) -> u16 {
    fn parse(b: &[u8]) -> Option<u16> {
        let little = b.get(..2)? == b"II";
        if !little && b.get(..2)? != b"MM" {
            return None;
        }
        let u16_at = |i: usize| {
            let a: [u8; 2] = b.get(i..i + 2)?.try_into().ok()?;
            Some(if little {
                u16::from_le_bytes(a)
            } else {
                u16::from_be_bytes(a)
            })
        };
        let u32_at = |i: usize| {
            let a: [u8; 4] = b.get(i..i + 4)?.try_into().ok()?;
            Some(if little {
                u32::from_le_bytes(a)
            } else {
                u32::from_be_bytes(a)
            })
        };
        if u16_at(2)? != 42 {
            return None;
        }
        let start = u32_at(4)? as usize;
        let count = usize::from(u16_at(start)?).min(4096);
        for n in 0..count {
            let k = start.checked_add(2 + n * 12)?;
            if u16_at(k)? == 0x112 && u16_at(k + 2)? == 3 && u32_at(k + 4)? == 1 {
                return u16_at(k + 8);
            }
        }
        None
    }
    let mut k = 2;
    while k + 4 <= bytes.len() && bytes[k] == 0xff {
        let marker = bytes[k + 1];
        if marker == 0xda || marker == 0xd9 {
            break;
        }
        let len = u16::from_be_bytes([bytes[k + 2], bytes[k + 3]]) as usize;
        if len < 2 || k + 2 + len > bytes.len() {
            break;
        }
        if marker == 0xe1 && bytes.get(k + 4..k + 10) == Some(b"Exif\0\0") {
            return parse(&bytes[k + 10..k + 2 + len]).unwrap_or(1);
        }
        k += 2 + len;
    }
    1
}
pub fn orient(src: Raster, orientation: u16) -> Raster {
    if !(2..=8).contains(&orientation) {
        return src;
    }
    let (w, h) = if orientation > 4 {
        (src.height, src.width)
    } else {
        (src.width, src.height)
    };
    let mut rgba = vec![0; src.rgba.len()];
    for y in 0..h {
        for x in 0..w {
            let (xx, yy) = match orientation {
                2 => (src.width - x - 1, y),
                3 => (src.width - x - 1, src.height - y - 1),
                4 => (x, src.height - y - 1),
                5 => (y, x),
                6 => (y, src.height - x - 1),
                7 => (src.width - y - 1, src.height - x - 1),
                8 => (src.width - y - 1, x),
                _ => unreachable!(),
            };
            let k = ((y * w + x) * 4) as usize;
            let m = ((yy * src.width + xx) * 4) as usize;
            rgba[k..k + 4].copy_from_slice(&src.rgba[m..m + 4]);
        }
    }
    Raster {
        width: w,
        height: h,
        rgba,
    }
}
pub fn encode(image: &Raster) -> Result<Vec<u8>, String> {
    dimensions(image.width, image.height)?;
    let mut out = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut out, image.width, image.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .map_err(|e| e.to_string())?
            .write_image_data(&image.rgba)
            .map_err(|e| e.to_string())?;
    }
    Ok(out)
}
/// Explicit preview-only reduction. Full diff PNG/RGBA is never resized.
pub fn preview(image: &Raster, png: &[u8]) -> Result<(String, bool), String> {
    let (bytes, reduced) = if png.len() <= MAX_PREVIEW_PNG {
        (png.to_vec(), false)
    } else {
        let ratio = (1_500_000.0 / (f64::from(image.width) * f64::from(image.height)))
            .sqrt()
            .min(1.0);
        let (w, h) = (
            (f64::from(image.width) * ratio).floor().max(1.0) as u32,
            (f64::from(image.height) * ratio).floor().max(1.0) as u32,
        );
        let mut rgba = Vec::with_capacity(w as usize * h as usize * 4);
        for y in 0..h {
            for x in 0..w {
                let xx = u64::from(x) * u64::from(image.width) / u64::from(w);
                let yy = u64::from(y) * u64::from(image.height) / u64::from(h);
                let k = ((yy * u64::from(image.width) + xx) * 4) as usize;
                rgba.extend_from_slice(&image.rgba[k..k + 4]);
            }
        }
        (
            encode(&Raster {
                width: w,
                height: h,
                rgba,
            })?,
            true,
        )
    };
    if bytes.len() > MAX_PREVIEW_PNG {
        return Err("Preview budget exceeded".into());
    }
    Ok((
        format!("![](data:image/png;base64,{})", STANDARD.encode(bytes)),
        reduced,
    ))
}
