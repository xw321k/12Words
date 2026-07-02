mod crypto;

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::Manager;

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

/// Generate a random password
#[tauri::command]
fn generate_password(length: u32, use_numbers: bool, use_symbols: bool) -> Result<String, String> {
    let mut chars = vec!['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm',
        'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
        'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M',
        'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z'];
    if use_numbers {
        chars.extend_from_slice(&['0', '1', '2', '3', '4', '5', '6', '7', '8', '9']);
    }
    if use_symbols {
        chars.extend_from_slice(&['!', '@', '#', '$', '%', '^', '&', '*', '(', ')',
            '-', '_', '+', '=', '{', '}', '[', ']', '|', ':', ';', ',', '.', '/', '?', '~', '`']);
    }

    if chars.is_empty() {
        return Err("至少需要选择字母以外的字符类型".to_string());
    }

    let len = length.max(4).min(128) as usize;
    let mut rng = rand::thread_rng();
    let password: String = (0..len).map(|_| {
        let idx = rng.gen_range(0..chars.len());
        chars[idx]
    }).collect();
    Ok(password)
}

/// Read and decrypt the vault from disk
#[tauri::command]
fn read_vault(app_handle: tauri::AppHandle, seed_hex: String) -> Result<Vec<crypto::VaultEntry>, String> {
    let path = vault_path(&app_handle);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let encrypted = std::fs::read(&path)
        .map_err(|e| format!("读取密码库失败: {}", e))?;
    let data = crypto::VaultData::from_encrypted(&seed_hex, &encrypted)?;
    Ok(data.entries)
}

/// Encrypt and write the vault to disk
#[tauri::command]
fn write_vault(app_handle: tauri::AppHandle, seed_hex: String, entries: Vec<crypto::VaultEntry>) -> Result<(), String> {
    let data = crypto::VaultData { entries };
    let encrypted = data.to_encrypted(&seed_hex)?;
    let path = vault_path(&app_handle);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("创建目录失败: {}", e))?;
    }
    std::fs::write(&path, &encrypted)
        .map_err(|e| format!("写入密码库失败: {}", e))?;
    Ok(())
}

/// Get vault file path in app data dir
fn vault_path(app_handle: &tauri::AppHandle) -> PathBuf {
    let dir = app_handle.path().app_data_dir()
        .expect("无法获取应用数据目录");
    dir.join("vault.encrypted")
}

/// Open a URL in the system browser
#[tauri::command]
fn open_url(url: String) -> Result<(), String> {
    std::process::Command::new("cmd")
        .args(["/c", "start", "", &url])
        .spawn()
        .map_err(|e| format!("打开链接失败: {}", e))?;
    Ok(())
}

/// Export vault file to a user-chosen location
#[tauri::command]
fn export_vault(app_handle: tauri::AppHandle, seed_hex: String) -> Result<String, String> {
    use tauri_plugin_dialog::DialogExt;

    let source = vault_path(&app_handle);
    if !source.exists() {
        return Err("密码库文件不存在，请先添加密码数据".to_string());
    }

    // Re-encrypt to ensure the file is fresh
    let encrypted = std::fs::read(&source)
        .map_err(|e| format!("读取密码库失败: {}", e))?;
    let data = crypto::VaultData::from_encrypted(&seed_hex, &encrypted)?;
    let re_encrypted = data.to_encrypted(&seed_hex)?;

    let dest = app_handle.dialog()
        .file()
        .add_filter("加密密码库", &["12words"])
        .set_file_name("vault_export.12words")
        .blocking_save_file();

    match dest {
        Some(path) => {
            let p = path.into_path().map_err(|e| format!("路径错误: {}", e))?;
            std::fs::write(&p, &re_encrypted)
                .map_err(|e| format!("导出失败: {}", e))?;
            Ok("导出成功".to_string())
        }
        None => Err("用户取消了导出".to_string()),
    }
}

/// Import vault file from a user-chosen location
#[tauri::command]
fn import_vault(app_handle: tauri::AppHandle, seed_hex: String) -> Result<Vec<crypto::VaultEntry>, String> {
    use tauri_plugin_dialog::DialogExt;

    let file = app_handle.dialog()
        .file()
        .add_filter("加密密码库", &["12words"])
        .blocking_pick_file();

    match file {
        Some(path) => {
            let p = path.into_path().map_err(|e| format!("路径错误: {}", e))?;
            let encrypted = std::fs::read(&p)
                .map_err(|e| format!("读取备份文件失败: {}", e))?;
            let data = crypto::VaultData::from_encrypted(&seed_hex, &encrypted)?;

            // Write to app data dir
            let dest = vault_path(&app_handle);
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("创建目录失败: {}", e))?;
            }
            std::fs::write(&dest, &encrypted)
                .map_err(|e| format!("写入密码库失败: {}", e))?;
            Ok(data.entries)
        }
        None => Err("用户取消了导入".to_string()),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
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
            generate_password,
            open_url,
            export_vault,
            import_vault,
            read_vault,
            write_vault,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
