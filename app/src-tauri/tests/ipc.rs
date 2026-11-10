// Lib and bin unit tests are disabled (see Cargo.toml): only tests/ targets receive the
// Windows manifest from build.rs that lets a Tauri-linked executable start.
use maddentool_lib::with_commands;
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::test::{INVOKE_KEY, get_ipc_response, mock_builder, mock_context, noop_assets};
use tauri::webview::InvokeRequest;

// The origin Tauri treats as local on this platform, matching its private `protocol::origin`.
// Any other origin is remote, and remote requests are refused before reaching a command.
const LOCAL_ORIGIN: &str = if cfg!(any(windows, target_os = "android")) {
    "http://tauri.localhost"
} else {
    "tauri://localhost"
};

fn invoke(cmd: &str) -> Result<Value, Value> {
    let app = with_commands(mock_builder())
        .build(mock_context(noop_assets()))
        .expect("mock app builds");
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .expect("mock webview builds");
    get_ipc_response(
        &webview,
        InvokeRequest {
            cmd: cmd.into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: LOCAL_ORIGIN.parse().expect("valid url"),
            body: InvokeBody::default(),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.to_string(),
        },
    )
    .map(|body| body.deserialize::<Value>().expect("json body"))
}

#[test]
fn get_app_info_is_reachable_through_ipc_as_camel_case() {
    let response = invoke("get_app_info").expect("command is registered");
    assert_eq!(
        response,
        json!({
            "appName": "MaddenTool",
            "appVersion": env!("CARGO_PKG_VERSION"),
            "game": "Madden 27",
            "platform": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
        })
    );
}

#[test]
fn unknown_commands_are_rejected() {
    // The exact message shows the command table refused it, not the remote-origin guard,
    // whose error would also satisfy a bare `is_err()`.
    assert_eq!(
        invoke("read_file"),
        Err(json!("Command read_file not found"))
    );
}

fn manifest(text: &str) -> Value {
    serde_json::from_str(text).expect("valid json")
}

// What the webview may do, as tauri-build 2.7.4 and tauri-utils 2.10.2 resolve it:
//
// - With `app.security.capabilities` empty, every `.json` or `.toml` file under `capabilities/`,
//   at any depth, is active. A file can hold one capability, a list, or `{ "capabilities": [] }`.
// - With it set, its inline capabilities and references are what is active instead.
//
// So this reads the same sources and accepts only the reviewed grant. Anything it cannot read as
// exactly that is a problem. Mock-context IPC does not load capabilities, so none of this shows
// enforcement. It shows which grants exist. The Windows native smoke shows enforcement.
const REVIEWED_CAPABILITY: &str = "default";

fn capability_entries(document: &Value) -> Vec<&Value> {
    match document {
        Value::Array(list) => list.iter().collect(),
        Value::Object(map) => match map.get("capabilities") {
            Some(Value::Array(list)) => list.iter().collect(),
            _ => vec![document],
        },
        _ => vec![document],
    }
}

fn reviewed_capability_problems(entry: &Value) -> Vec<String> {
    let mut problems = Vec::new();
    let mut expect = |field: &str, wanted: &Value, actual: Option<&Value>| {
        let actual = actual.unwrap_or(&Value::Null);
        if actual != wanted {
            problems.push(format!("{field} is {actual}, expected {wanted}"));
        }
    };
    expect("windows", &json!(["main"]), entry.get("windows"));
    expect(
        "permissions",
        &json!(["allow-get-app-info"]),
        entry.get("permissions"),
    );
    for absent in ["webviews", "remote", "platforms"] {
        expect(absent, &Value::Null, entry.get(absent));
    }
    if let Some(local) = entry.get("local") {
        expect("local", &json!(true), Some(local));
    }
    problems
}

fn files_under(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir).expect("readable capabilities folder") {
        let path = entry.expect("readable entry").path();
        if path.is_dir() {
            files.extend(files_under(&path));
        } else {
            files.push(path);
        }
    }
    files.sort();
    files
}

/// Every reason the capability sources are not exactly the one reviewed grant.
fn capability_problems(dir: &Path, config: &Value) -> Vec<String> {
    let mut problems = Vec::new();
    match &config["app"]["security"]["capabilities"] {
        Value::Null => {}
        Value::Array(list) if list.is_empty() => {}
        other => problems.push(format!(
            "tauri.conf.json selects capabilities itself, which replaces the files: {other}"
        )),
    }
    let mut identifiers = Vec::new();
    for path in files_under(dir) {
        let name = path.display();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            problems.push(format!(
                "{name}: only reviewed JSON capabilities may be here"
            ));
            continue;
        }
        let text = fs::read_to_string(&path).expect("readable capability");
        let Ok(document) = serde_json::from_str::<Value>(&text) else {
            problems.push(format!("{name}: not valid JSON"));
            continue;
        };
        for entry in capability_entries(&document) {
            let identifier = entry["identifier"].as_str().unwrap_or("<none>").to_string();
            if identifier != REVIEWED_CAPABILITY {
                problems.push(format!("{name}: unreviewed capability {identifier}"));
            } else {
                for problem in reviewed_capability_problems(entry) {
                    problems.push(format!("{name}: {problem}"));
                }
            }
            identifiers.push(identifier);
        }
    }
    if identifiers != [REVIEWED_CAPABILITY] {
        problems.push(format!(
            "expected only the {REVIEWED_CAPABILITY} capability, found {identifiers:?}"
        ));
    }
    problems
}

fn host_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn shipped_config() -> Value {
    manifest(include_str!("../tauri.conf.json"))
}

/// A private copy of the real capability folder, removed when the test ends.
struct CapabilityFixture(PathBuf);

impl CapabilityFixture {
    fn new(name: &str) -> Self {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("capabilities-{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("fixture folder");
        fs::copy(
            host_path("capabilities/default.json"),
            dir.join("default.json"),
        )
        .expect("copy the real capability");
        Self(dir)
    }

    fn write(&self, relative: &str, text: &str) -> &Self {
        let path = self.0.join(relative);
        fs::create_dir_all(path.parent().expect("parent")).expect("nested folder");
        fs::write(path, text).expect("fixture file");
        self
    }

    fn problems(&self) -> Vec<String> {
        capability_problems(&self.0, &shipped_config())
    }
}

impl Drop for CapabilityFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const EXTRA_GRANT: &str = r#"{
  "identifier": "extra",
  "windows": ["main"],
  "permissions": ["core:window:allow-set-title"]
}"#;

#[test]
fn main_window_capability_grants_only_app_info() {
    assert_eq!(
        capability_problems(&host_path("capabilities"), &shipped_config()),
        Vec::<String>::new()
    );
}

// What the build script actually parsed from disk, in Tauri's own words. This catches a source
// the walk above reads differently from Tauri.
#[test]
fn the_build_resolved_only_the_reviewed_capability() {
    let resolved: Value = serde_json::from_str(
        &fs::read_to_string(host_path("gen/schemas/capabilities.json"))
            .expect("tauri-build writes this before tests compile"),
    )
    .expect("valid json");
    let resolved = resolved.as_object().expect("capabilities by identifier");
    assert_eq!(
        resolved.keys().collect::<Vec<_>>(),
        [REVIEWED_CAPABILITY],
        "{resolved:?}"
    );
    assert_eq!(
        reviewed_capability_problems(&resolved[REVIEWED_CAPABILITY]),
        Vec::<String>::new()
    );
}

// Grants that bypass both the folder and the configuration.
#[test]
fn no_source_changes_how_capabilities_load() {
    for file in files_under(&host_path("src")) {
        let text = fs::read_to_string(&file).expect("readable source");
        assert!(!text.contains("add_capability"), "{}", file.display());
    }
    let build = fs::read_to_string(host_path("build.rs")).expect("readable build script");
    assert!(!build.contains("capabilities_path_pattern"));
}

#[test]
fn an_additional_capability_file_cannot_evade_the_check() {
    let fixture = CapabilityFixture::new("extra");
    assert_eq!(fixture.problems(), Vec::<String>::new());
    fixture.write("extra.json", EXTRA_GRANT);
    let problems = fixture.problems();
    assert!(
        problems
            .iter()
            .any(|p| p.contains("unreviewed capability extra")),
        "{problems:?}"
    );
}

#[test]
fn a_nested_capability_file_cannot_evade_the_check() {
    let fixture = CapabilityFixture::new("nested");
    fixture.write("beta/deep/extra.json", EXTRA_GRANT);
    let problems = fixture.problems();
    assert!(
        problems
            .iter()
            .any(|p| p.contains("unreviewed capability extra")),
        "{problems:?}"
    );
}

#[test]
fn every_capability_file_shape_is_read() {
    for (name, document) in [
        ("list", format!("[{EXTRA_GRANT}]")),
        ("named", format!(r#"{{"capabilities":[{EXTRA_GRANT}]}}"#)),
    ] {
        let fixture = CapabilityFixture::new(name);
        fixture.write("more.json", &document);
        let problems = fixture.problems();
        assert!(
            problems
                .iter()
                .any(|p| p.contains("unreviewed capability extra")),
            "{name}: {problems:?}"
        );
    }
}

#[test]
fn a_file_the_check_cannot_read_fails_closed() {
    for (name, relative, text) in [
        (
            "toml",
            "extra.toml",
            "identifier = 'extra'\npermissions = []",
        ),
        ("broken", "extra.json", "{ not json"),
    ] {
        let fixture = CapabilityFixture::new(name);
        fixture.write(relative, text);
        let problems = fixture.problems();
        assert!(
            problems.iter().any(|p| p.contains(relative)),
            "{name}: {problems:?}"
        );
    }
}

#[test]
fn broadening_the_reviewed_capability_is_caught() {
    for (name, document) in [
        (
            "permission",
            r#"{"identifier":"default","windows":["main"],"permissions":["allow-get-app-info","core:default"]}"#,
        ),
        (
            "window",
            r#"{"identifier":"default","windows":["*"],"permissions":["allow-get-app-info"]}"#,
        ),
        (
            "webview",
            r#"{"identifier":"default","windows":["main"],"webviews":["*"],"permissions":["allow-get-app-info"]}"#,
        ),
        (
            "remote",
            r#"{"identifier":"default","windows":["main"],"remote":{"urls":["https://*"]},"permissions":["allow-get-app-info"]}"#,
        ),
    ] {
        let fixture = CapabilityFixture::new(&format!("broad-{name}"));
        fixture.write("default.json", document);
        assert_ne!(fixture.problems(), Vec::<String>::new(), "{name}");
    }
}

#[test]
fn inline_or_selected_capabilities_in_the_config_are_caught() {
    for (name, entry) in [
        ("inline", manifest(EXTRA_GRANT)),
        ("reference", json!("default")),
    ] {
        let mut config = shipped_config();
        config["app"]["security"]["capabilities"] = json!([entry]);
        let problems = capability_problems(&host_path("capabilities"), &config);
        assert!(
            problems.iter().any(|p| p.contains("tauri.conf.json")),
            "{name}: {problems:?}"
        );
    }
}

#[test]
fn shipped_csp_is_local_only() {
    let config = manifest(include_str!("../tauri.conf.json"));
    let csp = config["app"]["security"]["csp"].as_str().expect("csp set");
    assert!(csp.contains("default-src 'none'"), "{csp}");
    for forbidden in ["unsafe-inline", "unsafe-eval", "https:", "http://*", "*"] {
        assert!(!csp.contains(forbidden), "{forbidden} in {csp}");
    }
}

#[test]
fn identity_and_paths_are_portable() {
    let config = manifest(include_str!("../tauri.conf.json"));
    assert_eq!(config["productName"], "MaddenTool");
    let identifier = config["identifier"].as_str().expect("identifier");
    assert!(!identifier.to_lowercase().contains("myfranchise"));
    // Absent, so Tauri uses the workspace Cargo.toml version.
    assert!(config.get("version").is_none());
    let dist = config["build"]["frontendDist"].as_str().expect("dist");
    assert_eq!(dist, "../../ui/dist");
}
