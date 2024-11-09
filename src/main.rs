use http_client::{client::Client, error::HttpError};
fn main() -> Result<(), HttpError> {
    let mut file = std::fs::File::create("fb.html")?;
    let client = Client::new();
    client
        .get("localhost:8080/Video.mp4")?
        .download_to_file(&mut file)?;

    Ok(())
}
