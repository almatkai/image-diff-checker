use image_diff_checker_native::codec;
use std::{fs, path::Path};

fn fixture(name: &str) -> Vec<u8> {
    fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("qa/fixtures/images")
            .join(name),
    )
    .unwrap()
}
fn segment(out: &mut Vec<u8>, marker: u8, bytes: &[u8]) {
    out.extend_from_slice(&[255, marker]);
    out.extend_from_slice(&((bytes.len() + 2) as u16).to_be_bytes());
    out.extend_from_slice(bytes);
}
// Independent minimal T.81 stream: one DC=0 code and one EOB code, both bit 0.
fn flat(w: u16, h: u16) -> Vec<u8> {
    let mut out = vec![255, 216];
    let mut q = vec![0];
    q.extend_from_slice(&[1; 64]);
    segment(&mut out, 219, &q);
    let mut sof = vec![8];
    sof.extend_from_slice(&h.to_be_bytes());
    sof.extend_from_slice(&w.to_be_bytes());
    sof.extend_from_slice(&[1, 1, 17, 0]);
    segment(&mut out, 192, &sof);
    for spec in [0, 16] {
        let mut table = vec![spec, 1];
        table.extend_from_slice(&[0; 15]);
        table.push(0);
        segment(&mut out, 196, &table);
    }
    segment(&mut out, 218, &[1, 1, 0, 0, 63, 0]);
    let blocks = usize::from(w).div_ceil(8) * usize::from(h).div_ceil(8);
    out.resize(out.len() + (blocks * 2).div_ceil(8), 0);
    let pad = (8 - blocks * 2 % 8) % 8;
    *out.last_mut().unwrap() |= ((1u16 << pad) - 1) as u8;
    out.extend_from_slice(&[255, 217]);
    out
}
#[test]
fn full_16mp_jpeg_and_header_bounds() {
    let decoded = codec::decode(&flat(4000, 4000)).unwrap();
    assert_eq!((decoded.width, decoded.height), (4000, 4000));
    assert!(decoded
        .rgba
        .chunks_exact(4)
        .all(|p| p == [128, 128, 128, 255]));
    for (w, h) in [(0, 1), (8193, 1), (4001, 4000), (8192, 8192)] {
        assert!(codec::decode(&flat(w, h)).is_err());
    }
}
#[test]
fn jpeg_truncation_tables_scans_coefficients_and_mutations() {
    for name in [
        "color-q95-s2.jpg",
        "progressive.jpg",
        "cmyk.jpg",
        "jpeg-blocks-33x19-restart1-p1.jpg",
    ] {
        let bytes = fixture(name);
        for end in 2..bytes.len() {
            assert!(
                codec::decode(&bytes[..end]).is_err(),
                "truncated {name}/{end}"
            );
        }
        // Damaged entropy/headers must always return a bounded result or error, never panic.
        for offset in 2..bytes.len() {
            let mut changed = bytes.clone();
            changed[offset] ^= 255;
            let result = codec::decode(&changed);
            if let Ok(im) = result {
                assert_eq!(im.rgba.len(), im.width as usize * im.height as usize * 4);
            }
        }
    }

    let scope = Path::new(env!("CARGO_MANIFEST_DIR")).join("qa/fixtures/jpeg-scope");
    let mut observations = serde_json::Map::new();
    for entry in fs::read_dir(scope).unwrap() {
        let path = entry.unwrap().path();
        let result = codec::decode(&fs::read(&path).unwrap());
        assert!(result.is_err(), "Narrow scope policy: {}", path.display());
        observations.insert(
            path.file_name().unwrap().to_str().unwrap().to_owned(),
            serde_json::json!({"nativeAccepted":false,"error":result.unwrap_err()}),
        );
    }
    assert_eq!(observations.len(), 5);
    fs::write(
        Path::new(env!("CARGO_MANIFEST_DIR")).join(".qa/jpeg-scope-native.json"),
        serde_json::to_vec_pretty(&observations).unwrap(),
    )
    .unwrap();
    let mut unusual = fixture("gray.jpg");
    let header = unusual.windows(2).position(|a| a == [255, 192]).unwrap();
    unusual[header + 1] = 193;
    unusual[header + 4] = 12;
    assert!(codec::decode(&unusual).is_err());
    let original = flat(17, 13);
    let dht = original.windows(2).position(|a| a == [255, 196]).unwrap();
    let sos = original.windows(2).position(|a| a == [255, 218]).unwrap();
    let sof = original.windows(2).position(|a| a == [255, 192]).unwrap();
    let mut malformed = vec![];
    let mut a = original.clone();
    a[dht + 5] = 3;
    malformed.push(("oversubscribed Huffman", a));
    let mut a = original.clone();
    a[dht + 4] = 32;
    malformed.push(("unknown Huffman class", a));
    let mut a = original.clone();
    a[dht + 21] = 12;
    malformed.push(("oversized DC category", a));
    let mut a = original.clone();
    a[sos + 8] = 64;
    malformed.push(("spectral range", a));
    let mut a = original.clone();
    a[sos + 9] = 255;
    malformed.push(("successive approximation", a));
    let mut a = original.clone();
    a[sof + 11] = 0;
    malformed.push(("zero sampling", a));
    let mut a = original.clone();
    a[sof + 10] = 2;
    malformed.push(("unknown selector", a));
    let mut a = original.clone();
    a[dht + 22 + 21] = 0xf1;
    malformed.push(("AC run outside block", a));
    let mut a = original.clone();
    a[dht + 22 + 21] = 0x0b;
    malformed.push(("oversized AC category", a));
    for (name, bytes) in malformed {
        assert!(codec::decode(&bytes).is_err(), "{name}");
    }
}
