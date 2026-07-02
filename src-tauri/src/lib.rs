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

/// Get vault file path — use app exe directory so it's portable with the exe
fn vault_path(_app_handle: &tauri::AppHandle) -> PathBuf {
    let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    let parent = exe.parent().unwrap_or(&PathBuf::from(".")).to_path_buf();
    parent.join("vault.encrypted")
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

/// Save mnemonic to a txt file in current working directory
#[tauri::command]
fn save_mnemonic_txt(phrase: String) -> Result<String, String> {
    let content = format!(
        "{}\n\n\
        建议将以上 12 个助记词物理抄写并保存在安全的地方，然后删除此文件。\n\
        这是恢复密码库的唯一方式，丢失后无法找回。\n",
        phrase
    );
    let path = std::env::current_dir()
        .map_err(|e| format!("获取目录失败: {}", e))?
        .join("12words_mnemonic.txt");
    std::fs::write(&path, &content)
        .map_err(|e| format!("写入文件失败: {}", e))?;
    Ok(path.to_string_lossy().to_string())
}

/// Export vault file — takes path + raw encrypted data, just writes
#[tauri::command]
fn export_vault(dest_path: String, encrypted_data: Vec<u8>) -> Result<String, String> {
    std::fs::write(&dest_path, &encrypted_data)
        .map_err(|e| format!("导出失败: {}", e))?;
    Ok("导出成功".to_string())
}

/// Import vault file — takes path + seed, reads decrypts and writes to app data
#[tauri::command]
fn import_vault_from(app_handle: tauri::AppHandle, source_path: String, seed_hex: String) -> Result<Vec<crypto::VaultEntry>, String> {
    let encrypted = std::fs::read(&source_path)
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

/// Read raw encrypted vault bytes
#[tauri::command]
fn read_vault_raw(app_handle: tauri::AppHandle) -> Result<Vec<u8>, String> {
    let path = vault_path(&app_handle);
    if !path.exists() {
        return Err("密码库文件不存在".to_string());
    }
    std::fs::read(&path)
        .map_err(|e| format!("读取失败: {}", e))
}

/// Check if vault file exists in app data dir
#[tauri::command]
fn vault_exists(app_handle: tauri::AppHandle) -> bool {
    vault_path(&app_handle).exists()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|_app, _argv, _cwd| {
            // Focus the existing window
        }))
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
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
            save_mnemonic_txt,
            vault_exists,
            export_vault,
            import_vault_from,
            read_vault_raw,
            read_vault,
            write_vault,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
