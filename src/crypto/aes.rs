use super::traits::{CryptoProvider, CryptoResult, CryptoError, KeyDerivation, Argon2KeyDerivation};
use std::io::{Read, Write};
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::{Aead, OsRng};
use rand::RngCore;

/// AES-256-GCM加密提供者
#[derive(Debug)]
pub struct AesCryptoProvider {
    key_derivation: Argon2KeyDerivation,
}

impl AesCryptoProvider {
    pub fn new() -> Self {
        Self {
            key_derivation: Argon2KeyDerivation,
        }
    }
}

impl CryptoProvider for AesCryptoProvider {
    fn algorithm_name(&self) -> &'static str {
        "AES-256-GCM"
    }
    
    fn encrypt_stream<R: Read, W: Write>(
        &self,
        password: &str,
        reader: &mut R,
        writer: &mut W,
    ) -> CryptoResult<()> {
        if password.is_empty() {
            return Err(CryptoError::InvalidPassword);
        }
        
        // 生成盐值
        let salt = self.key_derivation.generate_salt();
        
        // 派生密钥
        let key = self.key_derivation.derive_key(password, &salt)?;
        
        // 创建AES-GCM实例
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| CryptoError::EncryptionError(format!("AES密钥创建失败: {}", e)))?;
        
        // 写入盐值到文件头
        writer.write_all(&salt)?;
        
        // 分块加密
        let mut buffer = vec![0u8; self.chunk_size()];
        let mut chunk_index = 0u64;
        
        loop {
            let bytes_read = reader.read(&mut buffer)?;
            
            if bytes_read == 0 {
                break; // 文件读取完毕
            }
            
            // 生成随机nonce
            let mut nonce_bytes = [0u8; 12];
            OsRng.fill_bytes(&mut nonce_bytes);
            let nonce = Nonce::from_slice(&nonce_bytes);
            
            // 加密数据块
            let chunk = &buffer[0..bytes_read];
            let ciphertext = cipher.encrypt(nonce, chunk)
                .map_err(|e| CryptoError::EncryptionError(format!("加密失败 (块 {}): {}", chunk_index, e)))?;
            
            // 写入nonce和加密数据
            writer.write_all(&nonce_bytes)?;
            writer.write_all(&(ciphertext.len() as u32).to_le_bytes())?;
            writer.write_all(&ciphertext)?;
            
            chunk_index += 1;
        }
        
        Ok(())
    }
    
    fn decrypt_stream<R: Read, W: Write>(
        &self,
        password: &str,
        reader: &mut R,
        writer: &mut W,
    ) -> CryptoResult<()> {
        if password.is_empty() {
            return Err(CryptoError::InvalidPassword);
        }
        
        // 读取盐值
        let mut salt = vec![0u8; 32];
        reader.read_exact(&mut salt)?;
        
        // 派生密钥
        let key = self.key_derivation.derive_key(password, &salt)?;
        
        // 创建AES-GCM实例
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| CryptoError::DecryptionError(format!("AES密钥创建失败: {}", e)))?;
        
        // 分块解密
        let mut chunk_index = 0u64;
        
        loop {
            // 读取nonce
            let mut nonce_bytes = [0u8; 12];
            match reader.read_exact(&mut nonce_bytes) {
                Ok(_) => {},
                Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(CryptoError::IoError(e)),
            }
            let nonce = Nonce::from_slice(&nonce_bytes);
            
            // 读取数据长度
            let mut length_bytes = [0u8; 4];
            reader.read_exact(&mut length_bytes)?;
            let data_length = u32::from_le_bytes(length_bytes) as usize;
            
            // 读取加密数据
            let mut ciphertext = vec![0u8; data_length];
            reader.read_exact(&mut ciphertext)?;
            
            // 解密数据块
            let plaintext = cipher.decrypt(nonce, ciphertext.as_slice())
                .map_err(|e| CryptoError::DecryptionError(format!("解密失败 (块 {}): {}", chunk_index, e)))?;
            
            // 写入解密数据
            writer.write_all(&plaintext)?;
            
            chunk_index += 1;
        }
        
        Ok(())
    }
}

impl Default for AesCryptoProvider {
    fn default() -> Self {
        Self::new()
    }
} 