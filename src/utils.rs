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
