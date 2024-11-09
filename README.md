A simple HTTP client library written for educational purposes. 

# Example 1: Simple File Download

```Rust
use http_client::{client::Client, error::HttpError};
fn main() -> Result<(), HttpError> {
    let mut file = std::fs::File::create("fb.html")?;
    let client = Client::new();
    client
        .get("localhost:8080/Video.mp4")?
        .download_to_file(&mut file)?;

    Ok(())
}

```

# Example 2: Form Submission:
```Rust
use http_client::{client::Client, error::HttpError};
fn main() -> Result<(), HttpError> {
    let client = Client::new();
    client
        .post("localhost:8080/login_action")?
        .form_data("email", "test@mail.com")
        .form_data("password", "password")
        .send()?;
    Ok(())
}

```