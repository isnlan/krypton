pub mod traits;
pub mod aes;
pub mod chacha20;
pub mod engine;

pub use traits::{CryptoProvider, CryptoError, CryptoResult};
pub use engine::CryptoEngine;

use crate::models::EncryptionAlgorithm;
use std::io::{Read, Write};

/// 加密提供者枚举，解决trait对象安全问题
#[derive(Debug)]
pub enum CryptoProviderEnum {
    Aes(aes::AesCryptoProvider),
    ChaCha20(chacha20::ChaCha20CryptoProvider),
}

impl CryptoProvider for CryptoProviderEnum {
    fn algorithm_name(&self) -> &'static str {
        match self {
            CryptoProviderEnum::Aes(provider) => provider.algorithm_name(),
            CryptoProviderEnum::ChaCha20(provider) => provider.algorithm_name(),
        }
    }
    
    fn chunk_size(&self) -> usize {
        match self {
            CryptoProviderEnum::Aes(provider) => provider.chunk_size(),
            CryptoProviderEnum::ChaCha20(provider) => provider.chunk_size(),
        }
    }
    
    fn encrypt_stream<R: Read, W: Write>(
        &self,
        password: &str,
        reader: &mut R,
        writer: &mut W,
    ) -> CryptoResult<()> {
        match self {
            CryptoProviderEnum::Aes(provider) => provider.encrypt_stream(password, reader, writer),
            CryptoProviderEnum::ChaCha20(provider) => provider.encrypt_stream(password, reader, writer),
        }
    }
    
    fn decrypt_stream<R: Read, W: Write>(
        &self,
        password: &str,
        reader: &mut R,
        writer: &mut W,
    ) -> CryptoResult<()> {
        match self {
            CryptoProviderEnum::Aes(provider) => provider.decrypt_stream(password, reader, writer),
            CryptoProviderEnum::ChaCha20(provider) => provider.decrypt_stream(password, reader, writer),
        }
    }
    
    fn verify_password(&self, password: &str, data: &[u8]) -> CryptoResult<bool> {
        match self {
            CryptoProviderEnum::Aes(provider) => provider.verify_password(password, data),
            CryptoProviderEnum::ChaCha20(provider) => provider.verify_password(password, data),
        }
    }
}

/// 创建对应的加密提供者
pub fn create_crypto_provider(algorithm: &EncryptionAlgorithm) -> CryptoProviderEnum {
    match algorithm {
        EncryptionAlgorithm::AES256 => CryptoProviderEnum::Aes(aes::AesCryptoProvider::new()),
        EncryptionAlgorithm::ChaCha20 => CryptoProviderEnum::ChaCha20(chacha20::ChaCha20CryptoProvider::new()),
        EncryptionAlgorithm::Blowfish => {
            // 暂时返回AES作为占位符
            CryptoProviderEnum::Aes(aes::AesCryptoProvider::new())
        }
    }
}

/// 加密工具函数
pub fn encrypt_stream<R: Read, W: Write>(
    algorithm: &EncryptionAlgorithm,
    password: &str,
    reader: &mut R,
    writer: &mut W,
) -> CryptoResult<()> {
    let provider = create_crypto_provider(algorithm);
    provider.encrypt_stream(password, reader, writer)
}

/// 解密工具函数
pub fn decrypt_stream<R: Read, W: Write>(
    algorithm: &EncryptionAlgorithm,
    password: &str,
    reader: &mut R,
    writer: &mut W,
) -> CryptoResult<()> {
    let provider = create_crypto_provider(algorithm);
    provider.decrypt_stream(password, reader, writer)
} 