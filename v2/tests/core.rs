use image_diff_checker_native::{codec, files, threshold, validate_file, Adapter, COMMAND};
use serde_json::{json, Value};
use std::{fs, path::Path};
use tezbar_extension_sdk::{Extension, ExtensionPaths, TestContext};
#[test]
fn full_16_megapixel_decode_and_oversize_header() {
    let image = codec::Raster {
        width: 4000,
        height: 4000,
        rgba: vec![127; 64_000_000],
    };
    let png = codec::encode(&image).unwrap();
    let decoded = codec::decode(&png).unwrap();
    assert_eq!((decoded.width, decoded.height), (4000, 4000));
    assert_eq!(decoded.rgba, image.rgba);
    let mut oversize = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut oversize, 8193, 1);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(&vec![0; 8193 * 4])
            .unwrap();
    }
    assert!(codec::decode(&oversize).unwrap_err().contains("8192"));
}
fn owned() -> tempfile::TempDir {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join(".qa/unit");
    fs::create_dir_all(&p).unwrap();
    tempfile::tempdir_in(p).unwrap()
}
#[test]
fn source_validation_and_parsefloat() {
    for (input, value) in [
        ("0", 0.0),
        ("0.1", 0.1),
        (".5rest", 0.5),
        ("1e-1xx", 0.1),
        ("0.5e", 0.5),
        ("-1", 0.0),
        ("1e999", 1.0),
        ("Infinity", 1.0),
        ("-Infinity", 0.0),
        ("\u{feff} 0.2", 0.2),
    ] {
        assert_eq!(threshold(input).unwrap(), value, "{input}");
    }
    for input in [
        "inf",
        "NaN",
        "nan",
        "infinity",
        " ",
        "e1",
        "--1",
        "\u{85}0.2",
    ] {
        assert!(threshold(input).is_err(), "{input}");
    }
    assert_eq!(threshold("").unwrap_err(), "The threshold is required.");
    for name in [".png", "a.PNG", "a.jpeg", "x.JPG", "dotted.dir/a.gif"] {
        assert_eq!(validate_file(&json!([name])).unwrap(), name);
    }
    assert_eq!(
        validate_file(&json!([])).unwrap_err(),
        "The item is required."
    );
    assert_eq!(
        validate_file(&json!(["file.webp"])).unwrap_err(),
        "Invalid file type."
    );
}
#[test]
fn real_dimension_and_decode_boundaries() {
    assert!(codec::dimensions(4000, 4000).is_ok());
    assert!(codec::dimensions(8192, 1953).is_ok());
    for (w, h) in [(0, 1), (1, 0), (8193, 1), (4001, 4000), (8192, 8192)] {
        assert!(codec::dimensions(w, h).is_err());
    }
    assert!(codec::decode(&vec![0; codec::MAX_FILE + 1]).is_err());
    for bytes in [
        b"".as_slice(),
        b"GIF89a",
        b"\xff\xd8",
        b"\x89PNG\r\n\x1a\n",
        b"<svg onload=evil>",
    ] {
        assert!(codec::decode(bytes).is_err());
    }
}
#[test]
fn selected_file_regular_symlink_fifo_and_size() {
    let d = owned();
    let selected = d.path().join("x.PNG");
    fs::write(&selected, b"frozen").unwrap();
    assert_eq!(files::read_selected(&selected).unwrap(), b"frozen");
    assert_eq!(fs::read(&selected).unwrap(), b"frozen");
    assert!(files::read_selected(d.path()).is_err());
    assert!(files::read_selected(Path::new("relative.png")).is_err());
    let large = d.path().join("large.png");
    let f = fs::File::create(&large).unwrap();
    f.set_len(codec::MAX_FILE as u64).unwrap();
    assert_eq!(files::read_selected(&large).unwrap().len(), codec::MAX_FILE);
    f.set_len(codec::MAX_FILE as u64 + 1).unwrap();
    assert!(files::read_selected(&large).is_err());
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        let link = d.path().join("link.png");
        std::os::unix::fs::symlink(&selected, &link).unwrap();
        assert!(files::read_selected(&link).is_err());
        let dirlink = d.path().join("directory-link");
        std::os::unix::fs::symlink(d.path(), &dirlink).unwrap();
        assert!(files::read_selected(&dirlink.join("x.PNG")).is_err());
        let fifo = d.path().join("pipe.png");
        let c = std::ffi::CString::new(fifo.as_os_str().as_bytes()).unwrap();
        assert_eq!(unsafe { libc::mkfifo(c.as_ptr(), 0o600) }, 0);
        assert!(files::read_selected(&fifo).is_err());
    }
}
#[test]
fn bounded_preview_keeps_full_output_pixels() {
    let mut seed = 37u32;
    let rgba: Vec<_> = (0..1600 * 1600 * 4)
        .map(|_| {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (seed >> 24) as u8
        })
        .collect();
    let im = codec::Raster {
        width: 1600,
        height: 1600,
        rgba,
    };
    let png = codec::encode(&im).unwrap();
    assert!(png.len() > codec::MAX_PREVIEW_PNG);
    let (markdown, reduced) = codec::preview(&im, &png).unwrap();
    assert!(reduced);
    assert!(markdown.len() < 10_100_000);
    assert_eq!(codec::decode(&png).unwrap().rgba, im.rgba);
    use base64::Engine;
    let preview = base64::engine::general_purpose::STANDARD
        .decode(markdown.split(',').nth(1).unwrap().trim_end_matches(')'))
        .unwrap();
    let preview = codec::decode(&preview).unwrap();
    assert!(u64::from(preview.width) * u64::from(preview.height) <= 1_500_000);
}
#[tokio::test]
async fn canceled_open_checkpoints_prune_after_actual_adapter_entry() {
    let d = owned();
    let paths = ExtensionPaths {
        package_dir: Path::new(env!("CARGO_MANIFEST_DIR")).into(),
        data_dir: d.path().join("data"),
        temp_dir: d.path().join("temp"),
    };
    let adapter = Adapter::default();
    for n in 0..140 {
        let context = TestContext::new(paths.clone(), &[]);
        let mut state = json!({});
        let v = adapter
            .handle(
                &context.context,
                COMMAND,
                "extension:run-command",
                &json!({"sessionId":format!("abandoned-{n}")}),
                &mut state,
            )
            .await
            .unwrap();
        assert_eq!(v["root"]["type"], "Form");
        // TestContext::drop stops the actual captured SDK scope, after adapter entry.
        drop(context);
    }
    let context = TestContext::new(paths, &[]);
    let mut state = Value::Null;
    assert!(adapter
        .handle(
            &context.context,
            COMMAND,
            "extension:run-command",
            &json!({"sessionId":"after"}),
            &mut state
        )
        .await
        .is_ok());
    assert!(context.calls().is_empty());
    adapter.shutdown(&context.context).await.unwrap();
    assert!(!d.path().join("data").exists());
    assert!(!d.path().join("temp").exists());
}
