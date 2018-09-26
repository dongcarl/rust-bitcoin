//! # PSBT Serialization
//!
//! Defines traits used for (de)serializing PSBT values into/from raw
//! bytes in PSBT key-value pairs.

use std::io::{self, Cursor};

use secp256k1::{PublicKey, Secp256k1};

use blockdata::script::Script;
use blockdata::transaction::Transaction;
use consensus::encode::{self, serialize, Decodable};
use util::bip32::{ChildNumber, Fingerprint};

/// A trait for serializing a value as raw data for insertion into PSBT
/// key-value pairs.
pub trait Serialize {
    /// Serialize a value as raw data.
    fn serialize(&self) -> Vec<u8>;
}

/// A trait for deserializing a value from raw data in PSBT key-value pairs.
pub trait Deserialize: Sized {
    /// Deserialize a value from raw data.
    fn deserialize(bytes: &[u8]) -> Result<Self, encode::Error>;
}

impl_psbt_de_serialize!(Transaction);

impl Serialize for Script {
    fn serialize(&self) -> Vec<u8> {
        self.to_bytes()
    }
}

impl Deserialize for Script {
    fn deserialize(bytes: &[u8]) -> Result<Self, encode::Error> {
        Ok(Self::from(bytes.to_vec()))
    }
}

impl Serialize for PublicKey {
    fn serialize(&self) -> Vec<u8> {
        self.serialize().to_vec()
    }
}

impl Deserialize for PublicKey {
    fn deserialize(bytes: &[u8]) -> Result<Self, encode::Error> {
        PublicKey::from_slice(&Secp256k1::new(), bytes)
            .map_err(|_| encode::Error::ParseFailed("invalid public key"))
    }
}

impl Serialize for (Fingerprint, Vec<ChildNumber>) {
    fn serialize(&self) -> Vec<u8> {
        let mut rv: Vec<u8> = Vec::with_capacity(4 + 4 * (&self.1).len());

        rv.append(&mut self.0.to_bytes().to_vec());

        for cnum in self.1.iter() {
            rv.append(&mut serialize(&u32::from(cnum.clone())))
        }

        rv
    }
}

impl Deserialize for (Fingerprint, Vec<ChildNumber>) {
    fn deserialize(bytes: &[u8]) -> Result<Self, encode::Error> {
        if bytes.len() < 4 {
            return Err(io::Error::from(io::ErrorKind::UnexpectedEof).into())
        }

        let fprint: Fingerprint = Fingerprint::from(&bytes[0..4]);
        let mut dpath: Vec<ChildNumber> = Default::default();

        let d = &mut Cursor::new(&bytes[4..]);
        loop {
            match Decodable::consensus_decode(d) {
                Ok(index) => {
                    dpath.push(<ChildNumber as From<u32>>::from(index));

                    if d.position() == (bytes.len() - 4) as u64 {
                        break;
                    }
                },
                Err(e) => return Err(e),
            }
        }

        Ok((fprint, dpath))
    }
}
