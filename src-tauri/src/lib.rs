use std::fmt::{self, write};

use serde_json::json;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use tauri_plugin_store::StoreExt;

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    TauriShortcut(#[from] tauri_plugin_global_shortcut::Error),
    Tauri(#[from] tauri::Error),
}

// we must manually implement serde::Serialize
impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(app_handle: tauri::AppHandle, name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command(rename_all = "snake_case")]
fn add_shortcut(
    app_handle: tauri::AppHandle,
    path: &str,
    shortcut: &Shortcut,
) -> Result<(), Error> {
    let ctrl_n_shortcut = Shortcut::new(Some(Modifiers::CONTROL), Code::KeyN);
    app_handle.plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |_app, shortcut, event| {
                println!("{:?}", shortcut);
                if shortcut == &ctrl_n_shortcut {
                    match event.state() {
                        ShortcutState::Pressed => {
                            println!("Ctrl-N Pressed!");
                        }
                        ShortcutState::Released => {
                            println!("Ctrl-N Released!");
                        }
                    }
                }
            })
            .build(),
    )?;

    app_handle.global_shortcut().register(ctrl_n_shortcut)?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(desktop)]
    {
        tauri::Builder::default()
            .plugin(tauri_plugin_store::Builder::new().build())
            .plugin(tauri_plugin_global_shortcut::Builder::new().build())
            .plugin(tauri_plugin_opener::init())
            .setup(|app| {
                let store = app.store("settings.json")?;

                if store.get("commands").is_some() {
                    store.set("commands", json!([]));
                }

                Ok(())
            })
            .invoke_handler(tauri::generate_handler![greet])
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    }
}
