use yubikey_piv::key::{AlgorithmId, SlotId};
use yubikey_piv::certificate::PublicKeyInfo;

use crate::yubikey::management::{fetch_pubkey, sign_data};

use crate::ssh::{
    Curve,
    EcdsaPublicKey,
    KeyType,
    PublicKey,
    PublicKeyKind,
};

use crate::ssh::utils::asn_der_to_r_s;

/// Pull the public key from the YubiKey and wrap it in a Rustica
/// PublicKey object.
pub fn ssh_cert_fetch_pubkey(slot: SlotId) -> Option<PublicKey> {
    match fetch_pubkey(slot) {
        //Ok(hsm::PublicKeyInfo::Rsa { pubkey, .. }) => pubkey,
        Ok(PublicKeyInfo::EcP256(pubkey)) => {
            let key_type = KeyType::from_name("ecdsa-sha2-nistp256").unwrap();
            let curve = Curve::from_identifier("nistp256").unwrap();
            let kind = EcdsaPublicKey {
                curve,
                key: pubkey.as_bytes().to_vec(),
            };

            Some(PublicKey {
                key_type,
                kind: PublicKeyKind::Ecdsa(kind),
                comment: None,
            })
        },
        Ok(PublicKeyInfo::EcP384(pubkey)) => {
            let key_type = KeyType::from_name("ecdsa-sha2-nistp384").unwrap();
            let curve = Curve::from_identifier("nistp384").unwrap();
            let kind = EcdsaPublicKey {
                curve,
                key: pubkey.as_bytes().to_vec(),
            };

            Some(PublicKey {
                key_type,
                kind: PublicKeyKind::Ecdsa(kind),
                comment: None,
            })
        }
        _ => None,
    }
}

/// Sign the provided buffer of data and return it in an SSH Certificiate
/// signature formatted byte vector
pub fn ssh_cert_signer(buf: &[u8], slot: SlotId) -> Option<Vec<u8>> {
    match sign_data(&buf, AlgorithmId::EccP256, slot) {
        Ok(signature) => {
            let sig_type = "ecdsa-sha2-nistp256";
            let mut encoded: Vec<u8> = (sig_type.len() as u32).to_be_bytes().to_vec();
            encoded.extend_from_slice(sig_type.as_bytes());
            let (r,s) = match asn_der_to_r_s(&signature) {
                Some((r,s)) => (r, s),
                None => return None,
            };
            let mut sig_encoding = vec![];
            sig_encoding.extend_from_slice(&(r.len() as u32).to_be_bytes());
            sig_encoding.extend_from_slice(r);
            sig_encoding.extend_from_slice(&(s.len() as u32).to_be_bytes());
            sig_encoding.extend_from_slice(s);

            encoded.extend_from_slice(&(sig_encoding.len() as u32).to_be_bytes());
            encoded.extend(sig_encoding);

            Some(encoded)
        },
        Err(e) => {
            println!("Error: {:?}", e);
            None
        },
    }
}
