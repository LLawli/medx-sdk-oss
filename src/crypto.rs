/// Parseia chave pública RSA no formato XML do .NET (RSAKeyValue)
/// e encripta `data` usando OAEP com SHA-1 — mesmo esquema que
/// RSACryptoServiceProvider.Encrypt(data, true) no .NET.
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use num_bigint_dig::BigUint;
use rand::rngs::OsRng;
use rsa::{Oaep, RsaPublicKey};
use sha1::Sha1;

use crate::error::MedxError;

/// Extrai o conteúdo de uma tag XML simples (sem atributos, sem namespaces).
fn xml_text<'a>(xml: &'a str, tag: &str) -> Option<&'a str> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)? + start;
    Some(xml[start..end].trim())
}

/// Constrói `RsaPublicKey` a partir do XML no formato `<RSAKeyValue>`.
pub fn rsa_public_key_from_xml(xml: &str) -> Result<RsaPublicKey, MedxError> {
    let modulus_b64 = xml_text(xml, "Modulus")
        .ok_or_else(|| MedxError::KeyFormat("Modulus não encontrado no XML".into()))?;
    let exponent_b64 = xml_text(xml, "Exponent")
        .ok_or_else(|| MedxError::KeyFormat("Exponent não encontrado no XML".into()))?;

    let n = BigUint::from_bytes_be(&B64.decode(modulus_b64)?);
    let e = BigUint::from_bytes_be(&B64.decode(exponent_b64)?);

    RsaPublicKey::new(n, e).map_err(|e| MedxError::Rsa(e.to_string()))
}

/// Encripta `plaintext` com RSA-OAEP (SHA-1) e retorna o resultado em Base64.
pub fn rsa_oaep_encrypt(public_key: &RsaPublicKey, plaintext: &str) -> Result<String, MedxError> {
    let padding = Oaep::new::<Sha1>();
    let encrypted = public_key
        .encrypt(&mut OsRng, padding, plaintext.as_bytes())
        .map_err(|e| MedxError::Rsa(e.to_string()))?;
    Ok(B64.encode(&encrypted))
}

// ── Testes unitários ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Chave pública real retornada por /api/security/getkeys (512-bit RSA).
    const REAL_XML: &str = "<RSAKeyValue>\
        <Modulus>sF51htqKlsTjozS7s13RVIPc7wFTSc+6JoAQqCafmFTgOQ1w5WctkKuJ0M5HWlE08L8NSxjQipbcXzVCbPijIQ==</Modulus>\
        <Exponent>AQAB</Exponent>\
        </RSAKeyValue>";

    #[test]
    fn xml_text_extrai_conteudo_simples() {
        let xml = "<RSAKeyValue><Modulus>abc123</Modulus><Exponent>AQAB</Exponent></RSAKeyValue>";
        assert_eq!(xml_text(xml, "Modulus"), Some("abc123"));
        assert_eq!(xml_text(xml, "Exponent"), Some("AQAB"));
    }

    #[test]
    fn xml_text_retorna_none_para_tag_ausente() {
        let xml = "<RSAKeyValue><Exponent>AQAB</Exponent></RSAKeyValue>";
        assert!(xml_text(xml, "Modulus").is_none());
    }

    #[test]
    fn xml_text_trim_espacos() {
        let xml = "<RSAKeyValue><Modulus>  abc  </Modulus></RSAKeyValue>";
        assert_eq!(xml_text(xml, "Modulus"), Some("abc"));
    }

    #[test]
    fn rsa_key_from_xml_valido() {
        let key = rsa_public_key_from_xml(REAL_XML);
        assert!(key.is_ok(), "falha ao parsear chave: {:?}", key.err());
    }

    #[test]
    fn rsa_key_sem_modulus_retorna_erro() {
        let xml = "<RSAKeyValue><Exponent>AQAB</Exponent></RSAKeyValue>";
        let err = rsa_public_key_from_xml(xml).unwrap_err();
        assert!(matches!(err, crate::error::MedxError::KeyFormat(_)));
    }

    #[test]
    fn rsa_key_sem_exponent_retorna_erro() {
        let xml = "<RSAKeyValue><Modulus>sF51htqKlsTjozS7s13RVIPc7wFTSc+6JoAQqCafmFTgOQ1w5WctkKuJ0M5HWlE08L8NSxjQipbcXzVCbPijIQ==</Modulus></RSAKeyValue>";
        let err = rsa_public_key_from_xml(xml).unwrap_err();
        assert!(matches!(err, crate::error::MedxError::KeyFormat(_)));
    }

    #[test]
    fn rsa_key_base64_invalido_retorna_erro() {
        let xml = "<RSAKeyValue><Modulus>!!!não é base64!!!</Modulus><Exponent>AQAB</Exponent></RSAKeyValue>";
        let err = rsa_public_key_from_xml(xml).unwrap_err();
        assert!(matches!(err, crate::error::MedxError::Base64(_)));
    }

    #[test]
    fn encrypt_produz_base64_valido() {
        let key = rsa_public_key_from_xml(REAL_XML).unwrap();
        let enc = rsa_oaep_encrypt(&key, "Energia123").unwrap();
        // Base64 válido — decode não deve falhar
        assert!(B64.decode(&enc).is_ok());
    }

    #[test]
    fn encrypt_oaep_e_probabilistico() {
        // OAEP usa padding aleatório: dois encrypts da mesma string devem ser diferentes.
        let key = rsa_public_key_from_xml(REAL_XML).unwrap();
        let enc1 = rsa_oaep_encrypt(&key, "senha").unwrap();
        let enc2 = rsa_oaep_encrypt(&key, "senha").unwrap();
        assert_ne!(enc1, enc2, "OAEP deve ser probabilístico");
    }

    #[test]
    fn encrypt_tamanho_compativel_com_chave_512_bits() {
        let key = rsa_public_key_from_xml(REAL_XML).unwrap();
        let enc = rsa_oaep_encrypt(&key, "abc").unwrap();
        let bytes = B64.decode(&enc).unwrap();
        // Tamanho do ciphertext = tamanho do módulo = 64 bytes para 512-bit
        assert_eq!(bytes.len(), 64);
    }
}
