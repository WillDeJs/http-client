use http_parse::*;
use rustls::pki_types::ServerName;
use std::cmp::min;

use std::io::Cursor;
use std::marker::PhantomData;

use std::{fmt::Display, io::Write, net::TcpStream};

use crate::config::Config;
use crate::error::HttpError;

const LIB_USER_AGENT: &str = "HTTP Lib / 0.1.0 WD Client";
const MAX_BLOCK_SIZE: usize = 1_000_000;

pub struct Body;
pub struct NoBody;

/// A HTTP Client.
///
/// # Example 1:
/// ``` no_run
/// use http_client::{client::Client, error::HttpError};
/// fn main() -> Result<(), HttpError> {
///     let client = Client::new();
///     client
///         .post("localhost:8080/login_action")?
///         .form_data("email", "test@mail.com")
///         .form_data("password", "password")
///         .send()?;
///     Ok(())
/// }
/// ```
/// # Example 2:
/// ``` no_run
/// use http_client::{client::Client, error::HttpError};
/// fn main() -> Result<(), HttpError> {
///     let mut file = std::fs::File::create("video.mp4")?;
///     let client = Client::new();
///     client
///         .get("localhost:8080/Video.mp4")?
///         .download_to_file(&mut file)?;
///     Ok(())
/// }
/// ```
///
#[derive(Debug, Default)]
pub struct Client;

impl Client {
    /// Create a new Client
    pub fn new() -> Self {
        Self
    }

    /// Creates a new POST request to the given URL
    pub fn post(&self, url: &str) -> Result<ClientRequest<Body>, HttpError> {
        let url = HttpUrl::try_from(url).map_err(|e| HttpError::ParseError(e.to_string()))?;
        Ok(ClientRequest::new(url, HttpMethod::Post))
    }
    /// Creates a new GET request to the given URL
    pub fn get(&self, url: &str) -> Result<ClientRequest<NoBody>, HttpError> {
        let url = HttpUrl::try_from(url).map_err(|e| HttpError::InvalidUrl(e.to_string()))?;
        Ok(ClientRequest::new(url, HttpMethod::Get))
    }
    /// Creates a new HEAD request to the given URL
    pub fn head(&self, url: &str) -> Result<ClientRequest<NoBody>, HttpError> {
        let url = HttpUrl::try_from(url).map_err(|e| HttpError::InvalidUrl(e.to_string()))?;
        Ok(ClientRequest::new(url, HttpMethod::Head))
    }
    /// Creates a new PUT request to the given URL
    pub fn put(&self, url: &str) -> Result<ClientRequest<Body>, HttpError> {
        let url = HttpUrl::try_from(url).map_err(|e| HttpError::InvalidUrl(e.to_string()))?;
        Ok(ClientRequest::new(url, HttpMethod::Put))
    }
    /// Creates a new CONNECT request to the given URL
    pub fn connect(&self, url: &str) -> Result<ClientRequest<NoBody>, HttpError> {
        let url = HttpUrl::try_from(url).map_err(|e| HttpError::InvalidUrl(e.to_string()))?;
        Ok(ClientRequest::new(url, HttpMethod::Connect))
    }
    /// Creates a new TRACE request to the given URL
    pub fn trace(&self, url: &str) -> Result<ClientRequest<NoBody>, HttpError> {
        let url = HttpUrl::try_from(url).map_err(|e| HttpError::InvalidUrl(e.to_string()))?;
        Ok(ClientRequest::new(url, HttpMethod::Trace))
    }
    /// Creates a new PATCH request to the given URL
    pub fn patch(&self, url: &str) -> Result<ClientRequest<Body>, HttpError> {
        let url = HttpUrl::try_from(url).map_err(|e| HttpError::InvalidUrl(e.to_string()))?;
        Ok(ClientRequest::new(url, HttpMethod::Trace))
    }
    /// Creates a new OPTIONS request to the given URL
    pub fn options(&self, url: &str) -> Result<ClientRequest<NoBody>, HttpError> {
        let url = HttpUrl::try_from(url).map_err(|e| HttpError::InvalidUrl(e.to_string()))?;
        Ok(ClientRequest::new(url, HttpMethod::Trace))
    }
}

pub(crate) enum SecurityType {
    Tls,
    None,
}

pub(crate) enum FileSize {
    Sized(usize),
    Chunked,
}

pub struct ClientRequest<T> {
    url: HttpUrl,
    inner: HttpRequest,
    secure: bool,
    _d: PhantomData<T>,
}

impl ClientRequest<Body> {
    /// Add a data to the body of the request
    /// # Arguments
    /// `data`  data to be added
    pub fn body(mut self, data: &[u8]) -> Self {
        self.inner.add_data(data);
        self
    }
    /// Add a URL-Encoded data to the body of the request.
    /// # Arguments
    /// `name`  ID for the encoded entry being added
    /// `value` The value of entry being added.
    pub fn form_data(mut self, name: &str, value: impl Display) -> Self {
        self.inner.put_header(H_CONTENT_TYPE, MINE_URLENCODED_FORM);
        if self.inner.data().is_empty() {
            self.inner.add_data(format!("{name}={value}").as_bytes());
        } else {
            self.inner.add_data(format!("&{name}={value}").as_bytes());
        }
        self
    }
}
impl<T> ClientRequest<T> {
    /// Create a new ClientRequest
    /// # Argument
    /// `url`   URL being added
    /// `method`    HTTP Method used for creating the request.
    pub(crate) fn new(url: HttpUrl, method: HttpMethod) -> ClientRequest<T> {
        let secure = url.scheme().eq_ignore_ascii_case("https");
        ClientRequest {
            inner: HttpRequest::builder()
                .method(method)
                .path(url.path())
                .header(H_USER_AGENT, LIB_USER_AGENT)
                .header(H_HOST, url.host())
                .build(),
            url,
            secure,
            _d: PhantomData,
        }
    }

    /// Add a a header to this request.
    /// # Arguments
    /// `key`   Header name being added
    /// `value` The value of header being added.
    pub fn header(mut self, key: &str, value: impl Display) -> Self {
        self.inner.put_header(key, value);
        self
    }

    /// Send this request to the given given URL.
    pub fn send(self) -> Result<HttpResponse, HttpError> {
        Self::send_request(self.secure, &self.url, &self.inner)
    }

    /// Download the URL resource and return it's bytes.
    pub fn download(self) -> Result<Vec<u8>, HttpError> {
        match self.request_size() {
            Ok(_file_size) => match _file_size {
                FileSize::Sized(size) => {
                    let mut cursor = Cursor::new(Vec::with_capacity(size));
                    self.download_sized(size, &mut cursor)?;
                    Ok(cursor.into_inner())
                }
                FileSize::Chunked => self.download_chunked(),
            },
            Err(e) => Err(e),
        }
    }

    /// Download the URL resource and store the resource bytes.
    /// # Arguments
    /// `writer`    Destination for bytes sent by the remote server.
    pub fn download_to_file<V>(self, writer: &mut V) -> Result<(), HttpError>
    where
        V: Write,
    {
        match self.request_size() {
            Ok(_file_size) => match _file_size {
                FileSize::Sized(size) => {
                    self.download_sized(size, writer)?;
                    Ok(())
                }
                FileSize::Chunked => {
                    writer.write_all(&self.download_chunked()?)?;
                    Ok(())
                }
            },
            Err(e) => Err(e),
        }
    }

    /// Helper method, download chunked data from inner URL
    fn download_chunked(self) -> Result<Vec<u8>, HttpError> {
        Ok(self.send()?.data().to_owned())
    }

    /// Helper method, download sized data from inner URL
    fn download_sized<V>(mut self, size: usize, result: &mut V) -> Result<(), HttpError>
    where
        V: Write,
    {
        if size <= MAX_BLOCK_SIZE {
            let response = Self::send_request(self.secure, &self.url, &self.inner)?;
            result.write_all(response.data())?;
        } else {
            let mut start_byte = 0;
            let mut end_byte = size;
            let mut total_read = 0;
            while total_read < size {
                self.inner
                    .put_header(H_RANGE, format!("bytes={start_byte}-{end_byte}/{size}"));
                let response = Self::send_request(self.secure, &self.url, &self.inner)?;
                if response.status_code() != StatusCode::PARTIAL_CONTENT
                    && response.status_code() != StatusCode::OK
                {
                    return Err(HttpError::BadResponse(
                        response.status_code(),
                        response.status_msg().to_owned(),
                    ));
                }

                let header = response.header(H_CONTENT_RANGE);
                match header {
                    Some(header) => {
                        let tokens: Vec<usize> = header
                            .value::<String>()
                            .unwrap() // save to unwrap, a str can always turn into String
                            .replace("bytes", "")
                            .trim()
                            .split(['-', '/'])
                            .flat_map(|value| value.parse::<usize>())
                            .collect();
                        if tokens.len() < 3 {
                            return Err(HttpError::BadResponse(
                                response.status_code(),
                                format!("Unsupported value for header`{}`", header),
                            ));
                        }
                        total_read += tokens[1] - tokens[0] + 1;
                        start_byte = tokens[1] + 1;
                        end_byte = min(size, end_byte + MAX_BLOCK_SIZE);
                        result.write_all(response.data())?;
                    }
                    None => {
                        return Err(HttpError::BadResponse(
                            response.status_code(),
                            format!("Missing expected header: `{}`", H_CONTENT_RANGE),
                        ))
                    }
                }
            }
        }
        Ok(())
    }

    /// Helper method, send a request for the given URL
    fn send_request(
        secure: bool,
        url: &HttpUrl,
        req: &HttpRequest,
    ) -> Result<HttpResponse, HttpError> {
        if secure {
            Self::send_secure_request(url, req)
        } else {
            Self::send_insecure_request(url, req)
        }
    }
    /// Helper method, send a request for the given URL using a secure HTTP connection
    fn send_insecure_request(url: &HttpUrl, req: &HttpRequest) -> Result<HttpResponse, HttpError> {
        let mut connection = TcpStream::connect(url.address())?;
        connection.write_all(&req.into_bytes())?;
        let mut parser = HttpParser::from_reader(&mut connection);
        let response = match req.method() {
            HttpMethod::Head | HttpMethod::Connect => parser.response_head_only(),
            _ => parser.response(),
        }?;
        Ok(response)
    }
    /// Helper method, send a request for the given URL using a non-secure HTTP connection
    fn send_secure_request(url: &HttpUrl, req: &HttpRequest) -> Result<HttpResponse, HttpError> {
        let config = Config::tls_settings();
        let name = url.host().to_owned();
        let server_name =
            ServerName::try_from(name).map_err(|_e| HttpError::InvalidUrl(url.to_string()))?;
        let mut connection = rustls::ClientConnection::new(config, server_name)
            .map_err(|e| HttpError::ConnectionError(e.to_string()))?;
        let mut socket = TcpStream::connect(url.address())?;
        let mut tls = rustls::Stream::new(&mut connection, &mut socket);
        tls.write_all(&req.into_bytes())?;

        let mut parser = HttpParser::from_reader(&mut tls);
        let response = match req.method() {
            HttpMethod::Head | HttpMethod::Connect => parser.response_head_only(),
            _ => parser.response(),
        }?;
        Ok(response)
    }

    /// Helper method, retrieve the size of a remote resource being downloaded.
    fn request_size(&self) -> Result<FileSize, HttpError> {
        let mut request = HttpRequest::builder()
            .url(&self.url)
            .method(HttpMethod::Head)
            .build();
        for header in self.inner.headers() {
            request.put_header(header.name(), header.value::<String>().unwrap());
        }
        let response = Self::send_request(self.secure, &self.url, &request)?;
        if response.status_code() != StatusCode::OK {
            return Err(HttpError::BadResponse(
                response.status_code(),
                response.status_msg().to_owned(),
            ));
        }
        if let Some(header) = response.header(H_CONTENT_LENGTH) {
            return Ok(FileSize::Sized(header.value::<usize>()?));
        }
        let header = response.header(H_TRANSFER_ENCODING);
        if header.is_some() && header.unwrap().value::<String>().unwrap().contains("chunk") {
            Ok(FileSize::Chunked)
        } else {
            Err(HttpError::BadResponse(
                response.status_code(),
                format!(
                    "{}: Cannot determine resource size from headers.",
                    response.status_msg()
                ),
            ))
        }
    }
}
