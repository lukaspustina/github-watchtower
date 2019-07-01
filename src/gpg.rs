use crate::errors::*;

use failure::Fail;
use openpgp::{
    parse::{stream::*, Parse},
    TPK,
};
use sequoia_openpgp as openpgp;
use std::{io, path::Path};

#[derive(Debug)]
pub struct CommitVerifier {
    pub_keys: Vec<TPK>,
}

impl CommitVerifier {
    pub fn from_keys(pub_keys: Vec<TPK>) -> CommitVerifier {
        CommitVerifier { pub_keys }
    }

    pub fn from_key_file<P: AsRef<Path>>(file_path: P) -> Result<CommitVerifier> {
        let tpk = TPK::from_file(file_path).map_err(|e| e.context(ErrorKind::FailedToLoadKey))?;
        let keys = vec![tpk];

        Ok(CommitVerifier::from_keys(keys))
    }

    pub fn from_key_files<P: AsRef<Path>>(file_paths: &[P]) -> Result<CommitVerifier> {
        let mut keys = Vec::new();
        for p in file_paths {
            let tpk = TPK::from_file(p).map_err(|e| e.context(ErrorKind::FailedToLoadKey))?;
            keys.push(tpk);
        }

        Ok(CommitVerifier::from_keys(keys))
    }
}

impl CommitVerifier {
    pub fn verify_signature(self, message: &[u8], signature: &[u8]) -> Result<()> {
        let mut buffer = Vec::new();
        let mut verifier = DetachedVerifier::from_bytes(signature, message, self, None)
            .map_err(|e| e.context(ErrorKind::FailedToCreateVerifier))?;
        // Verify the data.
        io::copy(&mut verifier, &mut buffer).map_err(|e| e.context(ErrorKind::FailedToVerify))?;

        Ok(())
    }
}

impl VerificationHelper for CommitVerifier {
    fn get_public_keys(&mut self, _ids: &[openpgp::KeyID]) -> openpgp::Result<Vec<openpgp::TPK>> {
        Ok(self.pub_keys.clone())
    }

    fn check(&mut self, structure: &MessageStructure) -> openpgp::Result<()> {
        // In this function, we implement our signature verification policy.
        let mut good = false;
        for (i, layer) in structure.iter().enumerate() {
            match (i, layer) {
                // First, we are interested in signatures over the
                // data, i.e. level 0 signatures.
                (0, MessageLayer::SignatureGroup { ref results }) => {
                    // Finally, given a VerificationResult, which only says
                    // whether the signature checks out mathematically, we apply our policy.
                    match results.get(0) {
                        Some(VerificationResult::GoodChecksum(..)) => good = true,
                        Some(VerificationResult::MissingKey(_)) => {
                            return Err(failure::err_msg("Missing key to verify signature"))
                        }
                        Some(VerificationResult::BadChecksum(_)) => {
                            return Err(failure::err_msg("Bad signature"))
                        }
                        None => return Err(failure::err_msg("No signature")),
                    }
                }
                _ => return Err(failure::err_msg("Unexpected message structure")),
            }
        }

        if good {
            Ok(()) // Good signature.
        } else {
            Err(failure::err_msg("Signature verification failed"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use log::debug;
    use spectral::prelude::*;

    #[test]
    fn load_key() {
        let cv = CommitVerifier::from_key_file("tests/lukas.pustina.pub");

        asserting("Valid key has been successfully loaded")
            .that(&cv)
            .is_ok();
    }

    #[test]
    fn verify_static() {
        let signature = r#"-----BEGIN PGP SIGNATURE-----
Comment: GPGTools - http://gpgtools.org

iQIzBAABCAAdFiEEQWEMJmhTxtV/4Zdg7PtdAy2CkRIFAl0SFO8ACgkQ7PtdAy2C
kRIaMhAAmEk0z8JPOAbArXYyrLDabDAIFKpF6j0roW0QGqT7JueoJBMq73lXOzWJ
4g3OrxEMjGShyXP30w9NqUpyoXnucUUixc+IsbSBKQOq6FBM6wppiQbZKmli/XZS
f+4VUSe89eHAv55LkMTIp6TNNWEXTKWhEBoJcNoIMhGvSYuBLqbHCvph0Z5yjk8T
COO0KWKm7ZkZv2sFQdzbdqJIpH46v2wkZ/tLPn4whB130S3eDxW88YZ3fImEdhZ5
UGATBl01Aqf7BS3BSrk76TGUsd0X7/qG1GMl3UgwvgeoSeCuTSWWcjiU+wJBQfOt
/n8gl8k6kA6hyZg19FMyZ+stdAc8DRCW56pdjZIv8ugElRb8CllOODEMr2aBbpAz
34rBJRv1kpumc4LuxftvOwHZy+9Z346z35NTfG4bZFCIY7YIBU4jk8zvlQf2Psm5
I/D6Q0Vt07XO48iXr5GqOJVfohNIPPjhhu0YC7RRlo8wlVOxNJjvE7/3ZEi2OwwH
/j7B1mn5As36fQNL6uPpiBRozJwUthdGDYFzYjkaWnxN0aM3R0Fuff69xaSzJscr
MU8XXdTduX8SOGjIfjFG+aQ+eGggrv7Tbv5rwi3eUnhiVpx2A4y0SsepxyGtLojK
pD9KrYMWm/GPtMH876wYF035ePGemXIdGv1s1rC0ODQaJappORQ=
=eFK3
-----END PGP SIGNATURE-----"#;
        let payload = r#"tree d72ddcef503cc1542d0bc627579805f96f8aa101
parent 72cf6df73dbd1a13ac096319e00cb63e0f2846c7
author Lukas Pustina <lukas@pustina.de> 1561466095 +0200
committer Lukas Pustina <lukas@pustina.de> 1561466095 +0200

Github: add list endpoints
"#;
        let cv = CommitVerifier::from_key_file("tests/lukas.pustina.pub")
            .expect("failed to load public key");

        let res = cv.verify_signature(payload.as_ref(), signature.as_ref());
        debug!("Result: {:#?}", res);

        asserting("Signature is valid").that(&res).is_ok();
    }
}
