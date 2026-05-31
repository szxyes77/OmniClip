use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::Sha256;
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;

const SALT_FILE: &str = "omniclip_salt.bin";
const DEFAULT_PASSWORD: &str = "omniclip-local-default";

pub struct EncryptionManager {
    cipher: Aes256Gcm,
    data_dir: PathBuf,
}

impl EncryptionManager {
    pub fn new(app: &AppHandle) -> Result<Self, String> {
        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("OmniClip");

        if !data_dir.exists() {
            fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
        }

        let key = Self::derive_key(&data_dir, None)?;
        let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;
        Ok(Self { cipher, data_dir })
    }

    pub fn derive_key(
        data_dir: &PathBuf,
        master_password: Option<&str>,
    ) -> Result<[u8; 32], String> {
        let salt_path = data_dir.join(SALT_FILE);
        let salt = if salt_path.exists() {
            let salt_bytes = fs::read(&salt_path).map_err(|e| e.to_string())?;
            if salt_bytes.len() != 16 {
                return Err("Invalid salt file".to_string());
            }
            let mut arr = [0u8; 16];
            arr.copy_from_slice(&salt_bytes);
            arr
        } else {
            let mut s = [0u8; 16];
            rand::thread_rng().fill_bytes(&mut s);
            fs::write(&salt_path, &s).map_err(|e| e.to_string())?;
            s
        };

        let machine_fingerprint = Self::get_machine_fingerprint();
        let password_input = if let Some(pwd) = master_password {
            format!("{}::{}", machine_fingerprint, pwd)
        } else {
            format!("{}::{}", machine_fingerprint, DEFAULT_PASSWORD)
        };

        let mut key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(
            password_input.as_bytes(),
            &salt,
            100_000,
            &mut key,
        );

        Ok(key)
    }

    fn get_machine_fingerprint() -> String {
        #[cfg(target_os = "windows")]
        {
            if let Ok(output) = std::process::Command::new("wmic")
                .args(&["csproduct", "get", "UUID"])
                .output()
            {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let uuid = stdout.lines().nth(1).unwrap_or("").trim();
                if !uuid.is_empty() {
                    return uuid.to_string();
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Ok(output) = std::process::Command::new("ioreg")
                .args(&["-rd1", "-c", "IOPlatformExpertDevice"])
                .output()
            {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(line) = stdout.lines().find(|l| l.contains("IOPlatformUUID")) {
                    if let Some(uuid) = line.split('"').nth(3) {
                        return uuid.to_string();
                    }
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            if let Ok(contents) = fs::read_to_string("/etc/machine-id") {
                return contents.trim().to_string();
            }
        }

        "omniclip-fallback-fingerprint".to_string()
    }

    pub fn encrypt_string(&self, plaintext: &str) -> Result<String, String> {
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| e.to_string())?;

        let mut result = Vec::new();
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        Ok(hex_encode(&result))
    }

    pub fn decrypt_string(&self, encrypted: &str) -> Result<String, String> {
        let data = hex_decode(encrypted).map_err(|e| e.to_string())?;

        if data.len() < 12 {
            return Err("Invalid encrypted data".to_string());
        }

        let nonce = Nonce::from_slice(&data[..12]);
        let ciphertext = &data[12..];

        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| e.to_string())?;

        String::from_utf8(plaintext).map_err(|e| e.to_string())
    }

    pub fn encrypt_data_for_export(&self, data: &[u8]) -> Result<String, String> {
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(nonce, data)
            .map_err(|e| e.to_string())?;

        let mut result = Vec::new();
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        Ok(base64_encode(&result))
    }

    pub fn decrypt_data_for_import(&self, encrypted_b64: &str) -> Result<Vec<u8>, String> {
        let data = base64_decode(encrypted_b64).map_err(|e| e.to_string())?;

        if data.len() < 12 {
            return Err("Invalid encrypted import data".to_string());
        }

        let nonce = Nonce::from_slice(&data[..12]);
        let ciphertext = &data[12..];

        self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| e.to_string())
    }

    pub fn reencrypt_with_new_password(
        &self,
        old_encrypted: &str,
        new_key: &[u8; 32],
    ) -> Result<String, String> {
        let decrypted = self.decrypt_string(old_encrypted)?;
        let new_cipher = Aes256Gcm::new_from_slice(new_key).map_err(|e| e.to_string())?;

        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = new_cipher
            .encrypt(nonce, decrypted.as_bytes())
            .map_err(|e| e.to_string())?;

        let mut result = Vec::new();
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        Ok(hex_encode(&result))
    }
}

fn hex_encode(data: &[u8]) -> String {
    data.iter().fold(String::new(), |mut acc, b| {
        let _ = write!(acc, "{:02x}", b);
        acc
    })
}

fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
    let bytes = s.as_bytes();
    if bytes.len() % 2 != 0 {
        return Err("Invalid hex string".to_string());
    }

    (0..bytes.len())
        .step_by(2)
        .map(|i| {
            let high = hex_char_to_u4(bytes[i])?;
            let low = hex_char_to_u4(bytes[i + 1])?;
            Ok((high << 4) | low)
        })
        .collect()
}

fn hex_char_to_u4(c: u8) -> Result<u8, String> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'A'..=b'F' => Ok(c - b'A' + 10),
        _ => Err(format!("Invalid hex char: {}", c as char)),
    }
}

fn base64_encode(data: &[u8]) -> String {
    use std::fmt::Write;
    const CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    let chunks = data.chunks(3);
    for chunk in chunks {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        let _ = write!(result, "{}", CHARS[((triple >> 18) & 0x3F) as usize] as char);
        let _ = write!(result, "{}", CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            let _ = write!(result, "{}", CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            let _ = write!(result, "=");
        }
        if chunk.len() > 2 {
            let _ = write!(result, "{}", CHARS[(triple & 0x3F) as usize] as char);
        } else {
            let _ = write!(result, "=");
        }
    }
    result
}

fn base64_decode(s: &str) -> Result<Vec<u8>, String> {
    let s = s.trim_end_matches('=');
    let mut result = Vec::new();
    let bytes = s.as_bytes();
    for chunk in bytes.chunks(4) {
        if chunk.len() < 2 {
            return Err("Invalid base64".to_string());
        }
        let b0 = base64_char_to_u6(chunk[0])? as u32;
        let b1 = base64_char_to_u6(chunk[1])? as u32;
        let triple = (b0 << 6) | b1;
        result.push(((triple >> 4) & 0xFF) as u8);
        if chunk.len() > 2 {
            let b2 = base64_char_to_u6(chunk[2])? as u32;
            result.push(((triple << 2 | (b2 >> 4)) & 0xFF) as u8);
            if chunk.len() > 3 {
                let b3 = base64_char_to_u6(chunk[3])? as u32;
                result.push((((b2 & 0x0F) << 4) | (b3 & 0x3F)) as u8);
            }
        }
    }
    Ok(result)
}

fn base64_char_to_u6(c: u8) -> Result<u8, String> {
    match c {
        b'A'..=b'Z' => Ok(c - b'A'),
        b'a'..=b'z' => Ok(c - b'a' + 26),
        b'0'..=b'9' => Ok(c - b'0' + 52),
        b'+' => Ok(62),
        b'/' => Ok(63),
        _ => Err(format!("Invalid base64 char: {}", c as char)),
    }
}
