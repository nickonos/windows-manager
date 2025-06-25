use std::{
    fmt::{self},
    fs, io,
    path::{Path, PathBuf},
    sync::Arc,
};

use serde_json::json;
use tauri::Wry;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use tauri_plugin_store::{Store, StoreExt};

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    TauriShortcut(#[from] tauri_plugin_global_shortcut::Error),
    Tauri(#[from] tauri::Error),
    TauriStore(#[from] tauri_plugin_store::Error),
    TauriOpener(#[from] tauri_plugin_opener::Error),
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

#[tauri::command(rename_all = "snake_case")]
fn add_shortcut(app_handle: tauri::AppHandle) -> Result<(), Error> {
    let ctrl_n_shortcut = Shortcut::new(Some(Modifiers::CONTROL), Code::KeyB);

    if !app_handle.global_shortcut().is_registered(ctrl_n_shortcut) {
        app_handle.global_shortcut().register(ctrl_n_shortcut)?;
    }

    let store = get_store(&app_handle);
    let store_commands = store.get("commands").expect("Commands not found");
    let mut commands: Vec<String> =
        serde_json::from_value(store_commands).expect("Unable to parse commands");
    commands.push(ctrl_n_shortcut.into_string());
    store.set("commands", json![commands]);

    store.save()?;
    store.close_resource();
    Ok(())
}

fn get_store(app: &tauri::AppHandle) -> Arc<Store<Wry>> {
    app.store("settings.json")
        .expect("settings.json store not found")
}

fn find_all_executables(executables: &mut Vec<PathBuf>, path: &Path) -> io::Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path_buf = entry.path();
            if path_buf.is_dir() {
                find_all_executables(executables, &path_buf)?;
            } else if path_buf.extension().unwrap_or_default() == "exe" {
                executables.push(path_buf);
            } else if path_buf.extension().unwrap_or_default() == "lnk" {
                executables.push(path_buf);
            }
        }
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut executables: Vec<PathBuf> = Vec::new();
    find_all_executables(
        &mut executables,
        Path::new("C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs"),
    )
    .expect("something went wrong");
    println!("{:?}", executables);

    #[cfg(desktop)]
    {
        tauri::Builder::default()
            .plugin(tauri_plugin_dialog::init())
            .plugin(tauri_plugin_store::Builder::new().build())
            .plugin(tauri_plugin_opener::init())
            .setup(|app| {
                let store = app.store("settings.json")?;

                if store.get("commands").is_none() {
                    store.set("commands", json!([]));
                }

                let store_commands = store.get("commands").expect("Commands not found");
                let commands: Vec<String> =
                    serde_json::from_value(store_commands).expect("Unable to parse commands");

                app.handle()
                    .plugin(
                        tauri_plugin_global_shortcut::Builder::new()
                            .with_handler(move |_app, shortcut, event| {
                                println!("handle shortcut: {:?}", shortcut);

                                if commands.contains(&shortcut.into_string()) {
                                    match event.state() {
                                        ShortcutState::Pressed => {
                                            println!("{:?} pressed", shortcut.into_string());
                                        }
                                        ShortcutState::Released => {}
                                    }
                                }
                            })
                            .build(),
                    )
                    .expect("Unable to add handler");

                Ok(())
            })
            .invoke_handler(tauri::generate_handler![add_shortcut])
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    }
}
