use base64::{engine::general_purpose::STANDARD, Engine};
use image_diff_checker_native::{codec, pixelmatch};
use serde_json::{json, Value};
use std::{collections::HashMap, fs, path::Path};
fn bytes(v: &Value) -> Vec<u8> {
    STANDARD.decode(v.as_str().unwrap()).unwrap()
}
#[test]
fn locked_source_rgba_masks_counts_and_decoders() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let oracle: Value =
        serde_json::from_slice(&fs::read(root.join("qa/oracle.json")).unwrap()).unwrap();
    let mut source = HashMap::new();
    let mut native = HashMap::new();
    let mut decode = vec![];
    for im in oracle["images"].as_array().unwrap() {
        let name = im["path"].as_str().unwrap();
        let a = bytes(&im["rgba"]);
        let b =
            codec::decode(&fs::read(root.join("qa/fixtures/images").join(name)).unwrap()).unwrap();
        assert_eq!(
            (b.width, b.height),
            (
                im["width"].as_u64().unwrap() as u32,
                im["height"].as_u64().unwrap() as u32
            ),
            "{name}"
        );
        let channels = a.iter().zip(&b.rgba).filter(|(a, b)| a != b).count();
        let pixels = a
            .chunks_exact(4)
            .zip(b.rgba.chunks_exact(4))
            .filter(|(a, b)| a != b)
            .count();
        let max = a
            .iter()
            .zip(&b.rgba)
            .map(|(a, b)| a.abs_diff(*b))
            .max()
            .unwrap();
        assert_eq!(a, b.rgba, "Exact source decode required: {name}");
        decode.push(json!({"path":name,"dimensions":[b.width,b.height],"differentChannels":channels,"differentPixels":pixels,"maxChannelDelta":max}));
        source.insert(name.to_owned(), a);
        native.insert(name.to_owned(), b);
    }
    let mut raw_count = 0;
    for case in oracle["raw"].as_array().unwrap() {
        let (rgba, count) = pixelmatch::compare(
            &bytes(&case["a"]),
            &bytes(&case["b"]),
            case["w"].as_u64().unwrap() as u32,
            case["h"].as_u64().unwrap() as u32,
            case["threshold"].as_f64().unwrap(),
        )
        .unwrap();
        assert_eq!(rgba, bytes(&case["rgba"]), "raw case {raw_count}");
        assert_eq!(count, case["count"].as_u64().unwrap() as usize);
        raw_count += 1;
    }
    let mut representations = vec![];
    let mut pair_count = 0;
    for case in oracle["pairs"].as_array().unwrap() {
        let a = case["a"].as_str().unwrap();
        let b = case["b"].as_str().unwrap();
        let t = case["threshold"].as_f64().unwrap();
        let na = &native[a];
        let nb = &native[b];
        let expected = bytes(&case["rgba"]);
        let expected_count = case["count"].as_u64().unwrap() as usize;
        let (mask, count) =
            pixelmatch::compare(&source[a], &source[b], na.width, na.height, t).unwrap();
        assert_eq!(mask, expected, "Algorithm with Jimp pixels: {a}/{b}/{t}");
        assert_eq!(count, expected_count);
        let (actual, native_count) =
            pixelmatch::compare(&na.rgba, &nb.rgba, na.width, na.height, t).unwrap();
        let changed = actual
            .chunks_exact(4)
            .zip(expected.chunks_exact(4))
            .filter(|(a, b)| a != b)
            .count();
        let classes = actual
            .chunks_exact(4)
            .zip(expected.chunks_exact(4))
            .filter(|(a, b)| class(a) != class(b))
            .count();
        assert_eq!(
            actual, expected,
            "Actual decode/pixelmatch pipeline: {a}/{b}/{t}"
        );
        assert_eq!(
            native_count, expected_count,
            "Actual diff count: {a}/{b}/{t}"
        );
        let encoded = codec::encode(&codec::Raster {
            width: na.width,
            height: na.height,
            rgba: actual.clone(),
        })
        .unwrap();
        let decoded = codec::decode(&encoded).unwrap();
        assert_eq!((decoded.width, decoded.height), (na.width, na.height));
        assert_eq!(
            decoded.rgba, expected,
            "Actual decode/pixelmatch/PNG pipeline: {a}/{b}/{t}"
        );
        if changed > 0 || native_count != expected_count {
            representations.push(json!({"a":a,"b":b,"threshold":t,"differentOutputPixels":changed,"differentClassificationPixels":classes,"sourceCount":expected_count,"nativeCount":native_count}));
        }
        pair_count += 1;
    }
    let report = json!({"decode":decode,"pixelmatchRawExact":raw_count,"pixelmatchSourcePairsExact":pair_count,"nativeRepresentationDifferences":representations,"universalJPEGParity":false});
    fs::create_dir_all(root.join(".qa")).unwrap();
    fs::write(
        root.join(".qa/codec-report.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    assert!(report["nativeRepresentationDifferences"]
        .as_array()
        .unwrap()
        .is_empty());
    fs::write(
        root.join(".qa/jpeg-parity.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    println!(
        "{} source/{} raw exact masks; {} native representation differences",
        pair_count,
        raw_count,
        report["nativeRepresentationDifferences"]
            .as_array()
            .unwrap()
            .len()
    );
}
fn class(p: &[u8]) -> u8 {
    if p == [255, 0, 0, 255] {
        1
    } else if p == [255, 255, 0, 255] {
        2
    } else {
        0
    }
}
