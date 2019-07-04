pub(crate) mod sha256 {
    use crate::errors::*;

    use failure::Fail;
    use hex;
    use ring;
    use std::{fs::File, io::Read, path::Path};

    #[allow(dead_code)] // Not yet used anywhere but tests
    pub(crate) fn from_file<T: AsRef<Path>>(file_path: T) -> Result<Vec<u8>> {
        let mut file = File::open(file_path).map_err(|e| e.context(ErrorKind::GeneralError))?;
        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|e| e.context(ErrorKind::GeneralError))?;
        let sha = from_bytes(content.as_bytes());

        Ok(sha)
    }

    #[allow(dead_code)] // Not yet used anywhere but tests
    pub(crate) fn from_file_as_str<T: AsRef<Path>>(file_path: T) -> Result<String> {
        from_file(file_path).map(|sha| hex::encode(sha.as_slice()))
    }

    #[allow(dead_code)] // Not yet used anywhere but tests
    pub(crate) fn from_bytes_as_str(bytes: &[u8]) -> String {
        let sha = from_bytes(bytes);
        hex::encode(sha.as_slice())
    }

    #[allow(dead_code)] // Not yet used anywhere but tests
    pub(crate) fn from_bytes(bytes: &[u8]) -> Vec<u8> {
        ring::digest::digest(&ring::digest::SHA256, bytes)
            .as_ref()
            .to_vec()
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        use spectral::prelude::*;

        #[test]
        fn from_bytes_as_str_okay() {
            let expected =
                "bc2b31c66377bbf592cc11d409b2c44b02dee7f94f755af1417129f9831b377a".to_string();

            let shasum = from_bytes_as_str(b"Hello commits, how are you singed today?");

            asserting("shasum correctly computed")
                .that(&shasum)
                .is_equal_to(expected);
        }

        #[test]
        fn from_file_okay() {
            let expected =
                hex::decode("a4a76ada9d345a82ca8ff0e300388870340dee54cb83b2c3a1605452ce357b18")
                    .expect("Failed to decode hex representation of sha sum");

            let shasum = from_file("tests/config.toml");

            asserting("shasum correctly computed")
                .that(&shasum)
                .is_ok()
                .is_equal_to(&expected);
        }
    }
}

pub(crate) mod http {
    use crate::errors::*;

    use failure::Fail;
    use reqwest::{Response, StatusCode};

    pub(crate) trait GeneralErrHandler {
        type T: std::marker::Sized;

        fn general_err_handler(self, expected_status: StatusCode) -> Result<Self::T>;
    }

    impl GeneralErrHandler for Response {
        type T = Response;

        fn general_err_handler(mut self, expected_status: StatusCode) -> Result<Self> {
            match self.status() {
                code if code == expected_status => Ok(self),
                code @ StatusCode::UNAUTHORIZED => {
                    Err(Error::from(ErrorKind::ApiCallFailedInvalidToken(code)))
                }
                code @ StatusCode::TOO_MANY_REQUESTS => {
                    Err(Error::from(ErrorKind::ApiCallFailedTooManyRequests(code)))
                }
                _ => Err(handle_error(&mut self)),
            }
        }
    }

    fn handle_error(response: &mut Response) -> Error {
        let status_code = response.status();

        match response.text() {
            Ok(body) => Error::from(ErrorKind::ApiCallFailed(status_code, body)),
            Err(e) => e
                .context(ErrorKind::FailedToProcessHttpResponse(
                    status_code,
                    "reading body".to_string(),
                ))
                .into(),
        }
    }
}

#[cfg(test)]
pub(crate) mod test {
    use env_logger;

    pub(crate) fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }
}
