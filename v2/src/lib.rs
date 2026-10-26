pub mod codec;
pub mod files;
pub mod pixelmatch;
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    future::Future,
    path::Path,
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex,
    },
    task::{Context, Poll, Waker},
};
use tezbar_extension_sdk::{
    BoxFuture, CommandConcurrency, CoreResult, Extension, ExtensionContext,
};
pub const ID: &str = "com.tezbar.native.image-diff-checker";
pub const COMMAND: &str = "image-diff-check";
const MAX_SESSIONS: usize = 16;
const MAX_RETAINED: usize = 128 * 1024 * 1024;
#[derive(Default)]
pub struct Adapter {
    sessions: Mutex<HashMap<String, Session>>,
    sequence: AtomicU64,
}
struct ResultImage {
    png: Vec<u8>,
    markdown: String,
    reduced: bool,
}
struct Session {
    opened: ExtensionContext,
    token: u64,
    revision: u64,
    draft: Value,
    result: Option<ResultImage>,
    notice: String,
    errors: Value,
}
fn cancelled(ctx: &ExtensionContext) -> bool {
    let mut task = Context::from_waker(Waker::noop());
    matches!(
        Box::pin(ctx.cancelled()).as_mut().poll(&mut task),
        Poll::Ready(())
    )
}
fn lock<T>(m: &Mutex<T>) -> Result<std::sync::MutexGuard<'_, T>, String> {
    m.lock().map_err(|_| "Session lock failed".into())
}
fn node(kind: &str, props: Value) -> Value {
    json!({"type":kind,"props":props})
}
fn render(s: &Session) -> Value {
    let prefix = format!("f:{}:", s.token);
    let (root, actions) = if let Some(result) = &s.result {
        (
            node("Detail", json!({"markdown":result.markdown})),
            json!([{"id":format!("d:{}:back",s.token),"title":"Back","kind":"action"}]),
        )
    } else {
        let change = format!("{prefix}change");
        let mut children = vec![];
        for (id, title) in [("actual", "Actual"), ("expected", "Expected")] {
            children.push(node("Form.FilePicker",json!({"id":id,"title":title,"value":s.draft[id],"error":s.errors[id],"actionId":change,"allowMultipleSelection":false,"canChooseDirectories":false,"nativeAbsolutePath":true,"info":"Only files with the extensions .png, .jpeg, .jpg, and .gif are supported."})));
        }
        children.push(node("Form.Separator", json!({})));
        let choices: Vec<_> = (0..=10)
            .map(|n| {
                let v = if n == 10 {
                    "1".into()
                } else if n == 0 {
                    "0".into()
                } else {
                    format!("0.{n}")
                };
                node("Form.Dropdown.Item", json!({"value":v,"title":v}))
            })
            .collect();
        children.push(json!({"type":"Form.Dropdown","props":{"id":"threshold","title":"Threshold","value":s.draft["threshold"],"error":s.errors["threshold"],"actionId":change},"children":choices}));
        if !s.notice.is_empty() {
            children.push(node(
                "Form.Description",
                json!({"title":"Status","text":s.notice}),
            ));
        }
        (
            json!({"type":"Form","props":{"formToken":s.token.to_string(),"formRevision":s.revision},"children":children}),
            json!([{"id":format!("{prefix}submit"),"title":"Run","kind":"submit"}]),
        )
    };
    let mut v = tezbar_extension_sdk::view(
        "Image Diff Check",
        root,
        actions.as_array().unwrap().clone(),
    );
    v["refreshMs"] = Value::Null;
    if !s.notice.is_empty() {
        v["message"] = json!(s.notice);
    }
    if s.result.as_ref().is_some_and(|r| r.reduced) {
        v["message"]=json!("Success🎉 — preview reduced to fit the 16 MiB renderer frame. The full-resolution diff is retained until Back/close.");
    }
    v
}
fn fields(s: &mut Session, payload: &Value, change: bool) -> Result<bool, String> {
    let v = payload["formValues"]
        .as_object()
        .ok_or("Host must provide all current form fields")?;
    if v.get("__formToken").and_then(Value::as_str) != Some(&s.token.to_string()) {
        return Err("Stale form token".into());
    }
    let rev = v
        .get("__formRevision")
        .and_then(Value::as_u64)
        .filter(|r| *r <= 9_007_199_254_740_991)
        .ok_or("Host all-field revision prerequisite is required")?;
    if rev < s.revision || (change && rev == s.revision) {
        return Ok(false);
    }
    if v.keys().any(|k| {
        ![
            "actual",
            "expected",
            "threshold",
            "__formToken",
            "__formRevision",
        ]
        .contains(&k.as_str())
    }) {
        return Err("Unexpected form field".into());
    }
    let mut draft = json!({});
    for key in ["actual", "expected"] {
        let paths = v
            .get(key)
            .and_then(Value::as_array)
            .ok_or("Select one file per image")?;
        if paths.len() > 1
            || paths
                .iter()
                .any(|p| p.as_str().is_none_or(|s| s.len() > 4096))
        {
            return Err("Select one file per image, path at most 4096 bytes".into());
        }
        draft[key] = json!(paths);
    }
    let threshold = v
        .get("threshold")
        .and_then(Value::as_str)
        .filter(|v| v.len() <= 128)
        .ok_or("Threshold field must be bounded text")?;
    draft["threshold"] = json!(threshold);
    if rev == s.revision && draft != s.draft {
        return Err("Changed fields require a fresh form revision".into());
    }
    s.revision = rev;
    s.draft = draft;
    Ok(true)
}
/// JavaScript parseFloat accepts numeric prefixes (including incomplete exponent).
pub fn threshold(text: &str) -> Result<f64, String> {
    if text.is_empty() {
        return Err("The threshold is required.".into());
    }
    let text = text.trim_start_matches(|c:char| matches!(c, '\u{0009}'..='\u{000d}'|' '|'\u{00a0}'|'\u{1680}'|'\u{2000}'..='\u{200a}'|'\u{2028}'|'\u{2029}'|'\u{202f}'|'\u{205f}'|'\u{3000}'|'\u{feff}'));
    for p in ["Infinity", "+Infinity", "-Infinity"] {
        if text.starts_with(p) {
            return Ok(if p.starts_with('-') { 0.0 } else { 1.0 });
        }
    }
    for n in (1..=text.len()).rev().filter(|n| text.is_char_boundary(*n)) {
        if text[..n]
            .bytes()
            .all(|c| c.is_ascii_digit() || b"+-.eE".contains(&c))
        {
            if let Ok(v) = text[..n].parse::<f64>() {
                if !v.is_nan() {
                    return Ok(v.clamp(0.0, 1.0));
                }
            }
        }
    }
    Err("The threshold must be a number.".into())
}
pub fn validate_file(values: &Value) -> Result<&str, String> {
    let path = values[0].as_str().ok_or("The item is required.")?;
    let ext = path.rsplit('.').next().unwrap_or("").to_lowercase();
    if !["jpeg", "jpg", "png", "gif"].contains(&ext.as_str()) {
        return Err("Invalid file type.".into());
    }
    Ok(path)
}
fn validation_errors(draft: &Value) -> Value {
    let mut errors = json!({});
    for key in ["actual", "expected"] {
        if let Err(e) = validate_file(&draft[key]) {
            errors[key] = json!(e);
        }
    }
    if let Err(e) = threshold(draft["threshold"].as_str().unwrap()) {
        errors["threshold"] = json!(e);
    }
    errors
}
fn run(draft: &Value) -> Result<ResultImage, String> {
    let actual = validate_file(&draft["actual"])?;
    let expected = validate_file(&draft["expected"])?;
    let threshold = threshold(draft["threshold"].as_str().unwrap())?;
    let compare = || -> Result<ResultImage, String> {
        // Freeze both selected byte arrays before decoding/comparing. No source writes.
        let a = files::read_selected(Path::new(actual))?;
        let b = files::read_selected(Path::new(expected))?;
        let a = codec::decode(&a)?;
        let b = codec::decode(&b)?;
        if (a.width, a.height) != (b.width, b.height) {
            return Err("Image sizes don't match".into());
        }
        let (rgba, _count) = pixelmatch::compare(&a.rgba, &b.rgba, a.width, a.height, threshold)?;
        // V1 deliberately discards count/percentage and displays only the diff.
        let diff = codec::Raster {
            width: a.width,
            height: a.height,
            rgba,
        };
        let png = codec::encode(&diff)?;
        let (markdown, reduced) = codec::preview(&diff, &png)?;
        Ok(ResultImage {
            png,
            markdown,
            reduced,
        })
    };
    compare().map_err(|_| "An error occurred while comparing images.".into())
}
impl Adapter {
    fn next(&self) -> u64 {
        self.sequence.fetch_add(1, Ordering::Relaxed) + 1
    }
}
impl Extension for Adapter {
    fn id(&self) -> &'static str {
        ID
    }
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
    fn commands(&self) -> &'static [&'static str] {
        &[COMMAND]
    }
    fn concurrency(&self, _: &str) -> CommandConcurrency {
        CommandConcurrency::RejectWhileBusy
    }
    fn shutdown<'a>(&'a self, _: &'a ExtensionContext) -> BoxFuture<'a, CoreResult> {
        Box::pin(async move {
            lock(&self.sessions)?.clear();
            Ok(json!({}))
        })
    }
    fn handle<'a>(
        &'a self,
        ctx: &'a ExtensionContext,
        command: &'a str,
        operation: &'a str,
        payload: &'a Value,
        _state: &'a mut Value,
    ) -> BoxFuture<'a, CoreResult> {
        Box::pin(async move {
            let sid = payload["sessionId"]
                .as_str()
                .filter(|s| !s.is_empty() && s.len() <= 256)
                .ok_or("Invalid session ID")?;
            let mut sessions = lock(&self.sessions)?;
            if operation == "extension:dispose-session" {
                sessions.remove(sid);
                return Ok(json!({}));
            }
            if cancelled(ctx) {
                return Err("Operation cancelled".into());
            }
            sessions.retain(|_, s| !cancelled(&s.opened));
            if command != COMMAND {
                return Err("Unknown command".into());
            }
            if operation == "extension:run-command" {
                if sessions.len() >= MAX_SESSIONS || sessions.contains_key(sid) {
                    return Err("At most 16 comparison sessions may be open".into());
                }
                sessions.insert(
                    sid.into(),
                    Session {
                        opened: ctx.clone(),
                        token: self.next(),
                        revision: 0,
                        draft: json!({"actual":[],"expected":[],"threshold":"0"}),
                        result: None,
                        notice: String::new(),
                        errors: json!({}),
                    },
                );
                return Ok(render(&sessions[sid]));
            }
            let retained: usize = sessions
                .values()
                .filter_map(|s| s.result.as_ref())
                .map(|r| r.png.len() + r.markdown.len())
                .sum();
            let s = sessions
                .get_mut(sid)
                .ok_or("Session expired; reopen Image Diff Check")?;
            if operation == "extension:refresh-session" {
                return Ok(render(s));
            }
            if operation != "extension:invoke-action" {
                return Err("Unsupported operation".into());
            }
            let id = payload["actionId"]
                .as_str()
                .filter(|s| s.len() <= 80)
                .ok_or("Invalid action ID")?;
            if s.result.is_some() {
                if id != format!("d:{}:back", s.token)
                    || payload.get("formValues").is_some_and(|v| {
                        !v.is_null() && v.as_object().is_none_or(|o| !o.is_empty())
                    })
                {
                    return Ok(tezbar_extension_sdk::unchanged());
                }
                s.result = None;
                s.token = self.next();
                s.revision = 0;
                s.notice.clear();
                return Ok(render(s));
            }
            let change = id == format!("f:{}:change", s.token);
            if !change && id != format!("f:{}:submit", s.token) {
                return Ok(tezbar_extension_sdk::unchanged());
            }
            match fields(s, payload, change) {
                Ok(false) => return Ok(tezbar_extension_sdk::unchanged()),
                Err(e) => {
                    s.notice = e;
                    if !change {
                        s.token = self.next();
                        s.revision = 0;
                    }
                    return Ok(render(s));
                }
                Ok(true) => {}
            }
            if change {
                let errors = validation_errors(&s.draft);
                if let Some(old) = s.errors.as_object_mut() {
                    old.retain(|k, _| !errors[k].is_null());
                }
                s.notice.clear();
                return Ok(render(s));
            }
            // Authoritative checkpoint lives outside SDK's staged JSON state.
            s.token = self.next();
            s.revision = 0;
            s.errors = validation_errors(&s.draft);
            if !s.errors.as_object().unwrap().is_empty() {
                s.notice.clear();
                return Ok(render(s));
            }
            let result = run(&s.draft);
            if cancelled(ctx) {
                s.notice = "Comparison cancelled. Run again explicitly.".into();
                return Err(s.notice.clone());
            }
            match result {
                Ok(r) if retained + r.png.len() + r.markdown.len() <= MAX_RETAINED => {
                    s.result = Some(r);
                    s.notice = "Success🎉".into();
                }
                Ok(_) => {
                    s.notice =
                        "Comparison resource budget exceeded. Close another result and Run again."
                            .into()
                }
                Err(e) => s.notice = format!("Oops! Something went wrong: {e}"),
            }
            Ok(render(s))
        })
    }
}
