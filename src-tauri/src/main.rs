#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]




#[tauri::command]
fn hello(name: &str) -> String {
  format!("Hello {}!", name)
}


fn main() {
    tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![hello])
        .run(tauri::generate_context!())
        .expect("error while running tairu application");
}
