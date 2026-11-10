mod app_info;

/// The command surface, shared by the real app and the IPC tests.
pub fn with_commands<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    builder.invoke_handler(tauri::generate_handler![app_info::get_app_info])
}

pub fn run() {
    with_commands(tauri::Builder::default())
        .run(tauri::generate_context!())
        .expect("failed to run MaddenTool");
}
