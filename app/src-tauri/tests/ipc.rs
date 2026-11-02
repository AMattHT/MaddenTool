// Lib and bin unit tests are disabled (see Cargo.toml): only tests/ targets receive the
// Windows manifest from build.rs that lets a Tauri-linked executable start.
use maddentool_lib::with_commands;
use serde_json::{Value, json};
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

#[test]
fn main_window_capability_grants_only_app_info() {
    let capability = manifest(include_str!("../capabilities/default.json"));
    assert_eq!(capability["windows"], json!(["main"]));
    assert_eq!(capability["permissions"], json!(["allow-get-app-info"]));
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
