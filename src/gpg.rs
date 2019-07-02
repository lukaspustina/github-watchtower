use crate::{
    errors::*,
    github::commits::{Commit, Reason, Verification},
};

use log::{debug, trace, warn};
use openpgp::{
    parse::{stream::*, Parse},
    TPK,
};
use sequoia_openpgp as openpgp;
use std::path::Path;

#[derive(Debug)]
pub struct CommitVerifier {
    pub_keys: Vec<TPK>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct VerificationKey {
    finger_print: String,
    key_id: String,
    e_mails: Vec<String>,
}

impl From<TPK> for VerificationKey {
    fn from(tpk: TPK) -> VerificationKey {
        let finger_print = tpk.fingerprint().to_string();
        let key_id = tpk.fingerprint().to_keyid().to_string();
        let e_mails = tpk
            .userids()
            .map(|x| {
                x.userid()
                    .other_or_address()
                    .unwrap_or_else(|_| Some("<unknown>".to_string()))
            })
            .flatten()
            .collect();

        VerificationKey {
            finger_print,
            key_id,
            e_mails,
        }
    }
}

impl CommitVerifier {
    pub fn from_keys(pub_keys: Vec<TPK>) -> CommitVerifier {
        CommitVerifier { pub_keys }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<CommitVerifier> {
        let tpk = TPK::from_bytes(bytes).map_err(|e| e.context(ErrorKind::FailedToLoadKey))?;
        let keys = vec![tpk];

        Ok(CommitVerifier::from_keys(keys))
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

// I really don't like this side effect way to do things, but there doesn't seem to be another way
// to get the signature key out of `DetachedVerifier`. At least, the side effect is locally isolated
// in this method only.
impl CommitVerifier {
    pub fn verify(&self, commit: &Commit) -> Result<VerificationKey> {
        match commit.commit.verification {
            Verification {
                verified: true,
                reason: Reason::Valid,
                signature: Some(ref signature),
                payload: Some(ref message),
            } => verify_message(&self.pub_keys, message.as_ref(), signature.as_ref()),
            _ => Err(Error::from(ErrorKind::FailedToVerify(
                "commit verification object is invalid".to_string(),
            ))),
        }
    }
}

fn verify_message(pub_keys: &[TPK], message: &[u8], signature: &[u8]) -> Result<VerificationKey> {
    let mut result_key: Vec<TPK> = Vec::new();
    let vc = VerificationContext::new(pub_keys, &mut result_key);
    let _ = DetachedVerifier::from_bytes(signature, message, vc, None).map_err(|e| {
        e.context(ErrorKind::FailedToVerify(
            "signature could not be verified".to_string(),
        ))
    })?;

    if result_key.len() > 1 {
        warn!("Found multiple signing keys. That's not inherently bad, but unexpected and odd.")
    }

    let tpk = result_key.pop().ok_or_else(|| {
        Error::from(ErrorKind::FailedToVerify(
            "no key found; this should not happen".to_string(),
        ))
    })?;

    let key = tpk.into();
    debug!("Message successfully verified with key {:?}", key);

    Ok(key)
}

struct VerificationContext<'a> {
    pub_keys: &'a [TPK],
    result_key: &'a mut Vec<TPK>,
}

impl<'a> VerificationContext<'a> {
    pub fn new(pub_keys: &'a [TPK], result_key: &'a mut Vec<TPK>) -> VerificationContext<'a> {
        VerificationContext {
            pub_keys,
            result_key,
        }
    }
}

impl<'a> VerificationHelper for VerificationContext<'a> {
    fn get_public_keys(&mut self, _ids: &[openpgp::KeyID]) -> openpgp::Result<Vec<openpgp::TPK>> {
        Ok(self.pub_keys.to_vec())
    }

    fn check(&mut self, structure: &MessageStructure) -> openpgp::Result<()> {
        // In this function, we implement our signature verification policy.
        // It might seem odd that there is a mutable state variable `good` instread of an early
        // return in case of GoodChecksum, but there might be multiple signatures, so we need to
        // continue.
        let mut good = false;
        for (i, layer) in structure.iter().enumerate() {
            match (i, layer) {
                // First, we are interested in signatures over the
                // data, i.e. level 0 signatures.
                (0, MessageLayer::SignatureGroup { ref results }) => {
                    // Finally, given a VerificationResult, which only says
                    // whether the signature checks out mathematically, we apply our policy.
                    match results.get(0) {
                        Some(VerificationResult::GoodChecksum(_, tpk, _, _, _)) => {
                            trace!("Verfified with key: {:#?}", tpk);
                            self.result_key.push((*tpk).clone());
                            good = true;
                        }
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
            Ok(())
        } else {
            Err(failure::err_msg("Signature verification failed"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{github::commits::*, utils::test};

    use log::debug;
    use spectral::prelude::*;

    #[test]
    fn load_key_from_file() {
        test::init();

        let cv = CommitVerifier::from_key_file("tests/lukas.pustina.pub");

        asserting("Valid key has been successfully loaded")
            .that(&cv)
            .is_ok();
    }

    #[test]
    fn load_key_from_bytes() {
        test::init();

        let key_str = include_str!("../tests/lukas.pustina.pub");
        let cv = CommitVerifier::from_bytes(key_str.as_ref());

        asserting("Valid key has been successfully loaded")
            .that(&cv)
            .is_ok();
    }

    mod internal {
        use super::*;

        #[test]
        fn verify_message_okay() {
            test::init();

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
            let message = r#"tree d72ddcef503cc1542d0bc627579805f96f8aa101
parent 72cf6df73dbd1a13ac096319e00cb63e0f2846c7
author Lukas Pustina <lukas@pustina.de> 1561466095 +0200
committer Lukas Pustina <lukas@pustina.de> 1561466095 +0200

Github: add list endpoints
"#;
            let expected_key = VerificationKey {
                finger_print: "4161 0C26 6853 C6D5 7FE1  9760 ECFB 5D03 2D82 9112".to_string(),
                key_id: "ECFB 5D03 2D82 9112".to_string(),
                e_mails: vec![
                    "lukas.pustina@centerdevice.com".to_string(),
                    "lukas.pustina@codecentric.de".to_string(),
                    "lukas@pustina.de".to_string(),
                ],
            };

            let cv = CommitVerifier::from_key_file("tests/lukas.pustina.pub")
                .expect("failed to load public key");
            let res = super::verify_message(&cv.pub_keys, message.as_ref(), signature.as_ref());

            debug!("Res: {:#?}", res);
            asserting("Signature is valid")
                .that(&res)
                .is_ok()
                .is_equal_to(&expected_key);
        }

        #[test]
        fn verify_message_with_multuple_keys_okay() {
            test::init();

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
            let message = r#"tree d72ddcef503cc1542d0bc627579805f96f8aa101
parent 72cf6df73dbd1a13ac096319e00cb63e0f2846c7
author Lukas Pustina <lukas@pustina.de> 1561466095 +0200
committer Lukas Pustina <lukas@pustina.de> 1561466095 +0200

Github: add list endpoints
"#;
            let expected_key = VerificationKey {
                finger_print: "4161 0C26 6853 C6D5 7FE1  9760 ECFB 5D03 2D82 9112".to_string(),
                key_id: "ECFB 5D03 2D82 9112".to_string(),
                e_mails: vec![
                    "lukas.pustina@centerdevice.com".to_string(),
                    "lukas.pustina@codecentric.de".to_string(),
                    "lukas@pustina.de".to_string(),
                ],
            };

            let cv = CommitVerifier::from_key_files(&[
                "tests/lukas.pustina.pub",
                "tests/lukas.pustina-invalid.pub",
            ])
            .expect("failed to load public key");
            let res = super::verify_message(&cv.pub_keys, message.as_ref(), signature.as_ref());

            debug!("Res: {:#?}", res);
            asserting("Signature is valid")
                .that(&res)
                .is_ok()
                .is_equal_to(&expected_key);
        }

        #[test]
        #[should_panic(expected = r#"Signature verification failed"#)]
        fn verify_message_failed_bad_signature() {
            test::init();

            let signature = r#"no such signature"#;
            let message = r#"tree d72ddcef503cc1542d0bc627579805f96f8aa101
parent 72cf6df73dbd1a13ac096319e00cb63e0f2846c7
author Lukas Pustina <lukas@pustina.de> 1561466095 +0200
committer Lukas Pustina <lukas@pustina.de> 1561466095 +0200

Github: add list endpoints
"#;
            let cv = CommitVerifier::from_key_file("tests/lukas.pustina.pub")
                .expect("failed to load public key");

            let res = super::verify_message(&cv.pub_keys, message.as_ref(), signature.as_ref());

            debug!("Res: {:#?}", res);
            asserting("Signature is valid").that(&res).is_ok();
            asserting("Signature is not verified").that(&res).is_err();
            res.unwrap();
        }

        #[test]
        #[should_panic(expected = r#"Signature verification failed"#)]
        fn verify_message_failed_invalid_signature() {
            test::init();

            let signature = r#"-----BEGIN PGP SIGNATURE-----
Comment: GPGTools - http://gpgtools.org

TQIzBAABCAAdFiEEQWEMJmhTxtV/4Zdg7PtdAy2CkRIFAl0SFO8ACgkQ7PtdAy2C
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
            let message = r#"tree d72ddcef503cc1542d0bc627579805f96f8aa101
parent 72cf6df73dbd1a13ac096319e00cb63e0f2846c7
author Pustina Lukas <lukas@pustina.de> 1561466095 +0200
committer Lukas Pustina <lukas@pustina.de> 1561466095 +0200

Github: add list endpoints
"#;
            let cv = CommitVerifier::from_key_file("tests/lukas.pustina.pub")
                .expect("failed to load public key");

            let res = super::verify_message(&cv.pub_keys, message.as_ref(), signature.as_ref());

            debug!("Res: {:#?}", res);
            asserting("Signature is not verified").that(&res).is_err();
            res.unwrap();
        }

        #[test]
        #[should_panic(expected = r#"Missing key to verify signature"#)]
        fn verify_message_invalid_key() {
            test::init();

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
            let message = r#"tree d72ddcef503cc1542d0bc627579805f96f8aa101
parent 72cf6df73dbd1a13ac096319e00cb63e0f2846c7
author Lukas Pustina <lukas@pustina.de> 1561466095 +0200
committer Lukas Pustina <lukas@pustina.de> 1561466095 +0200

Github: add list endpoints
"#;
            let cv = CommitVerifier::from_key_file("tests/lukas.pustina-invalid.pub")
                .expect("failed to load public key");

            let res = super::verify_message(&cv.pub_keys, message.as_ref(), signature.as_ref());

            debug!("Res: {:#?}", res);
            asserting("Signature is not verified").that(&res).is_err();
            res.unwrap();
        }
    }

    #[test]
    fn verify_commit() {
        test::init();

        let commit = Commit {
            sha: Sha::new("72cf6df73dbd1a13ac096319e00cb63e0f2846c7".to_string()),
            commit: CommitDetail {
                author: PersonDetails {
                    name: "Lukas Pustina".to_string(),
                    email: "lukas@pustina.de".to_string(),
                    date: "2019-06-25T08:37:21+00:00".parse().unwrap(),
                },
                committer: PersonDetails {
                    name: "Lukas Pustina".to_string(),
                    email: "lukas@pustina.de".to_string(),
                    date: "2019-06-25T10:27:51+00:00".parse().unwrap(),
                },
                message: "Add travis config".to_string(),
                verification: Verification {
                    verified: true,
                    reason: Reason::Valid,
                    signature: Some(
                        "-----BEGIN PGP SIGNATURE-----\nComment: GPGTools - http://gpgtools.org\n\niQIzBAABCAAdFiEEQWEMJmhTxtV/4Zdg7PtdAy2CkRIFAl0R9ysACgkQ7PtdAy2C\nkRKdzQ//cDyI9JX93+c/893g8TDLAIYyoLqbBL700wSjXEMO7WLkXYOJtFMO8jlA\nKjecVo+v2b0Eq7t8xAWrGPXGYyCdrbqIJg6eQRWaSkrS9PwIwrWcraPcduvWPHk2\n7bxCykiuXe+R01+00zMICZY0P0WnvuaoZo4kL7s6etgGY3sQff+fXUI8sGg8KN1Y\nav+t+bGKJnONa+BomLuIMNUuh29DaDytB2N/xuvhE3Pj/WEiYDDlhh3Wka7nTmsM\nxMhaK8+Jjjsv9rhzW63yPKrc4tHLUHLjvs3f8bPZbSgZqvS6YpY2/Nm7l20N4HBy\nxwUQ1Ee6YaE6GS6InXUEcoLZu0DxvOP476r1VZ/l6t2YTkcvYp7yi1zHIF3AuVQs\nA9gb4gK0aI7uyKrbT86XJCKAeu1CuOIpp6fGwD39maD1LgB6tYoIiFj8kOHxM0cp\nlCRdM+rF5Sgmr5UYaaEpFM6uWvQ7O7SJWn4j1FwQN6Ul++1CUQjoq8XczXQhZ9e0\n7bzOF+KlahNUWElxCiatiBsKGAhZEVzHp4LALJQE5s7X/Ea1fqkF+c87+0FQXGUT\nV5YwhHK6LTutfgxVqyCUlK3pshFxyEkHb2zKQsoIr02KWbZH8uTzs56xNHCJ6mI/\nANFLOdKLkRWNBARGMAuiM2hTyEUUOL0F9uSQMMzRQTlrkL3lWRA=\n=ivRW\n-----END PGP SIGNATURE-----".to_string(),
                    ),
                    payload: Some(
                        "tree ea7435f6d72196332c436474a42aea8ce030d424\nparent c255ad2347d00cae3dd2d7a21e1357e50413fc4f\nauthor Lukas Pustina <lukas@pustina.de> 1561451841 +0200\ncommitter Lukas Pustina <lukas@pustina.de> 1561458471 +0200\n\nAdd travis config\n".to_string(),
                    ),
                },
            },
        };
        let expected_key = VerificationKey {
            finger_print: "4161 0C26 6853 C6D5 7FE1  9760 ECFB 5D03 2D82 9112".to_string(),
            key_id: "ECFB 5D03 2D82 9112".to_string(),
            e_mails: vec![
                "lukas.pustina@centerdevice.com".to_string(),
                "lukas.pustina@codecentric.de".to_string(),
                "lukas@pustina.de".to_string(),
            ],
        };

        let cv = CommitVerifier::from_key_file("tests/lukas.pustina.pub")
            .expect("failed to load public key");
        let res = cv.verify(&commit);

        debug!("Res: {:#?}", res);
        asserting("Signature is valid")
            .that(&res)
            .is_ok()
            .is_equal_to(&expected_key);
    }
}
