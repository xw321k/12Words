mod crypto;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MnemonicResult {
    pub phrase: String,
    pub seed_hex: String,
    pub user_id: String,
}

/// Generate a 12-word BIP39 mnemonic and derive seed + user_id
#[tauri::command]
fn generate_mnemonic() -> Result<MnemonicResult, String> {
    let phrase = crypto::generate_mnemonic()?;
    let seed_hex = crypto::mnemonic_to_seed(&phrase)?;
    let user_id = crypto::derive_user_id(&seed_hex);
    Ok(MnemonicResult {
        phrase,
        seed_hex,
        user_id,
    })
}

/// Validate a mnemonic phrase without consuming it
#[tauri::command]
fn validate_mnemonic(phrase: String) -> Result<bool, String> {
    crypto::validate_mnemonic(&phrase).map(|_| true)
}

/// Derive seed and user_id from an existing mnemonic (for re-import)
#[tauri::command]
fn import_mnemonic(phrase: String) -> Result<MnemonicResult, String> {
    let seed_hex = crypto::mnemonic_to_seed(&phrase)?;
    let user_id = crypto::derive_user_id(&seed_hex);
    Ok(MnemonicResult {
        phrase,
        seed_hex,
        user_id,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            generate_mnemonic,
            validate_mnemonic,
            import_mnemonic,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
