use chacha20poly1305::{aead::{Aead, KeyInit}, ChaCha20Poly1305, Nonce, Key};
use base64::{engine::general_purpose, Engine as _};

// Ключ має бути рівно 32 байти, nonce — рівно 12 байт.
const SECRET_KEY: &[u8; 32] = b"14)d_d34xd_f3sg!fxcv_dawe_fzxce_";
const NONCE_BYTES: &[u8; 12] = b"WfC234FeZr)N";

/// Приймає звичайний текст -> Шифрує -> Повертає зашифрований Base64-рядок
pub fn encrypt_string(plaintext: &str) -> Result<String, String> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(SECRET_KEY));
    let nonce = Nonce::from_slice(NONCE_BYTES);

    // 1. Перетворюємо вхідний рядок у байти та шифруємо
    let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| format!("Помилка шифрування: {:?}", e))?;

    // 2. Кодуємо зашифровані байти в читабельний Base64-рядок
    Ok(general_purpose::STANDARD.encode(ciphertext))
}

/// Приймає зашифрований Base64-рядок -> Розшифровує -> Повертає оригінальний текст
pub fn decrypt_string(base64_ciphertext: &str) -> Result<String, String> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(SECRET_KEY));
    let nonce = Nonce::from_slice(NONCE_BYTES);

    // 1. Декодуємо Base64-рядок назад у зашифровані байти
    let encrypted_bytes = general_purpose::STANDARD.decode(base64_ciphertext)
        .map_err(|e| format!("Невалідний Base64: {:?}", e))?;

    // 2. Розшифровуємо байти
    let decrypted_bytes = cipher.decrypt(nonce, encrypted_bytes.as_slice())
        .map_err(|e| format!("Помилка дешифрування (ключ або дані пошкоджено): {:?}", e))?;

    // 3. Перетворюємо розшифровані байти назад у нормальний Rust String
    String::from_utf8(decrypted_bytes)
        .map_err(|e| format!("Помилка конвертації в UTF-8 рядок: {:?}", e))
}
