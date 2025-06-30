use std::{
    any::Any, collections::HashMap, fmt::{self}, fs, io, path::{Path, PathBuf}, str::FromStr, sync::Arc
};

use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use tauri::{Wry};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use tauri_plugin_store::{Store, StoreExt};

const SETTINGS_FILE: &str = "settings.json";
const START_MENU_PATH: &str = "C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs";
type CommandsValue = HashMap<String, String>;

enum StoreKey {
    Commands,
}

impl StoreKey {
    fn as_str(&self) -> &'static str {
        match self {
            StoreKey::Commands => "commands"
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    TauriShortcut(#[from] tauri_plugin_global_shortcut::Error),
    Tauri(#[from] tauri::Error),
    TauriStore(#[from] tauri_plugin_store::Error),
    TauriOpener(#[from] tauri_plugin_opener::Error),
    SerdeJson(#[from] serde_json::Error)
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

fn get_store(app: &tauri::AppHandle) -> Arc<Store<Wry>> {
    app.store(SETTINGS_FILE)
        .expect(format!("{SETTINGS_FILE} store not found").as_str())
}


fn get_from_store<T: DeserializeOwned>(store: &Arc<Store<Wry>>, key: StoreKey) -> Result<Option<T>, serde_json::Error> {
    let value = store.get(key.as_str());
    if value.is_none(){
        return Ok(None)   
    }
    
    match serde_json::from_value(value.expect("invalid json value")) {
        Ok(val) => {
            Ok(Some(val))
        }
        Err(e) => {
            Err(e)
        }
    }
}

fn set_store_value<T: serde::Serialize>(store: &Arc<Store<Wry>>, key: StoreKey, value: T) { 
    store.set(key.as_str(), serde_json::to_value(value).expect("unable to serialize to json"));
}

#[tauri::command(rename_all = "snake_case")]
fn add_shortcut(app_handle: tauri::AppHandle, shortcut_str: String, path: String) -> Result<(), Error> {
    let ctrl_n_shortcut = Shortcut::new(Some(Modifiers::CONTROL), Code::KeyB);
    let parsed_shortcut = Shortcut::from_str(&shortcut_str);
    if parsed_shortcut.is_err() {
        return Err(
            Error::TauriShortcut(
                tauri_plugin_global_shortcut::Error::from(
                    parsed_shortcut.unwrap_err())
                )
            );
    }

    let shortcut = parsed_shortcut.unwrap();
    if !app_handle.global_shortcut().is_registered(shortcut) {
        app_handle.global_shortcut().register(shortcut)?;
    }

    let store = get_store(&app_handle);
    let mut commands : Vec<String> = get_from_store(&store, StoreKey::Commands)?.unwrap_or_default();
    commands.push(ctrl_n_shortcut.into_string());
    set_store_value(&store, StoreKey::Commands, commands);

    store.save()?;
    store.close_resource();
    Ok(())
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
        Path::new(START_MENU_PATH),
    )
    .expect("something went wrong");

    #[cfg(desktop)]
    {
        tauri::Builder::default()
            .plugin(tauri_plugin_dialog::init())
            .plugin(tauri_plugin_store::Builder::new().build())
            .plugin(tauri_plugin_opener::init())
            .setup(|app| {
                use tauri::Manager;

                let store = app.store(SETTINGS_FILE)?;
                println!("{:?}", app.path().app_data_dir());
                let commands : Option<CommandsValue> = get_from_store(&store, StoreKey::Commands)?;
                if commands.is_none() {
                    set_store_value(&store, StoreKey::Commands, HashMap::<&str, &str>::new());
                }

                // let commands = commands.unwrap_or_default();

                // app.handle()
                //     .plugin(
                //         tauri_plugin_global_shortcut::Builder::new()
                //             .with_handler(move |_app, shortcut, event| {
                //                 let application = commands.get(&shortcut.into_string());

                //                 if application.is_some(){
                //                     match event.state() {
                //                         ShortcutState::Pressed => {
                //                             println!("{:?} execute", application.unwrap());
                //                         }
                //                         ShortcutState::Released => {}
                //                     }
                //                 }
                //             })
                //             .build(),
                //     )
                //     .expect("Unable to add handler");

                Ok(())
            })
            .invoke_handler(tauri::generate_handler![add_shortcut])
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    }
}
