use std::io::{Read, Write};
use std::fmt;

/// 加密操作结果类型
pub type CryptoResult<T> = Result<T, CryptoError>;

/// 加密错误类型
#[derive(Debug)]
pub enum CryptoError {
    IoError(std::io::Error),
    EncryptionError(String),
    DecryptionError(String),
    KeyDerivationError(String),
    InvalidPassword,
    InvalidFormat,
}

impl fmt::Display for CryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CryptoError::IoError(e) => write!(f, "IO错误: {}", e),
            CryptoError::EncryptionError(msg) => write!(f, "加密错误: {}", msg),
            CryptoError::DecryptionError(msg) => write!(f, "解密错误: {}", msg),
            CryptoError::KeyDerivationError(msg) => write!(f, "密钥派生错误: {}", msg),
            CryptoError::InvalidPassword => write!(f, "密码无效"),
            CryptoError::InvalidFormat => write!(f, "文件格式无效"),
        }
    }
}

impl std::error::Error for CryptoError {}

impl From<std::io::Error> for CryptoError {
    fn from(error: std::io::Error) -> Self {
        CryptoError::IoError(error)
    }
}

/// 通用加密提供者接口
pub trait CryptoProvider: Send + Sync {
    /// 获取算法名称
    fn algorithm_name(&self) -> &'static str;
    
    /// 获取推荐的分块大小
    fn chunk_size(&self) -> usize {
        1024 * 1024 // 默认1MB
    }
    
    /// 加密数据流
    fn encrypt_stream<R: Read, W: Write>(
        &self,
        password: &str,
        reader: &mut R,
        writer: &mut W,
    ) -> CryptoResult<()>;
    
    /// 解密数据流
    fn decrypt_stream<R: Read, W: Write>(
        &self,
        password: &str,
        reader: &mut R,
        writer: &mut W,
    ) -> CryptoResult<()>;
    
    /// 验证密码（可选实现）
    fn verify_password(&self, _password: &str, _data: &[u8]) -> CryptoResult<bool> {
        Ok(true) // 默认实现总是返回true
    }
}

/// 密钥派生工具trait
pub trait KeyDerivation {
    /// 从密码派生密钥
    fn derive_key(&self, password: &str, salt: &[u8]) -> CryptoResult<Vec<u8>>;
    
    /// 生成随机盐值
    fn generate_salt(&self) -> Vec<u8>;
}

/// 默认的Argon2密钥派生实现
#[derive(Debug)]
pub struct Argon2KeyDerivation;

impl KeyDerivation for Argon2KeyDerivation {
    fn derive_key(&self, password: &str, salt: &[u8]) -> CryptoResult<Vec<u8>> {
        use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
        
        let argon2 = Argon2::default();
        let password_bytes = password.as_bytes();
        let salt_string = SaltString::encode_b64(salt)
            .map_err(|e| CryptoError::KeyDerivationError(format!("盐值编码失败: {}", e)))?;
        
        let password_hash = argon2.hash_password(password_bytes, &salt_string)
            .map_err(|e| CryptoError::KeyDerivationError(format!("密钥派生失败: {}", e)))?;
        
        let binding = password_hash.hash.unwrap();
        let key_bytes = binding.as_bytes();
        Ok(key_bytes[0..32].to_vec()) // 返回32字节密钥
    }
    
    fn generate_salt(&self) -> Vec<u8> {
        use rand::RngCore;
        use aes_gcm::aead::OsRng;
        
        let mut salt = vec![0u8; 32];
        OsRng.fill_bytes(&mut salt);
        salt
    }
} 