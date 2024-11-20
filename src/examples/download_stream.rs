use std::io::Read;

use http_client::{client::Client, error::HttpError};
use http_parse::{HttpUrl, StatusCode};

fn get_playlist_url_list(url: &str) -> Result<Vec<HttpUrl>, HttpError> {
    let client = Client::new();
    let res = client.get(url)?.send()?;
    if res.status_code() != StatusCode::OK {
        return Err(HttpError::BadResponse(
            res.status_code(),
            res.status_msg().to_owned(),
        ));
    }

    // In case the url does not contain a full URL.
    let url_base = match url.rfind("/") {
        Some(index) => &url[0..index],
        None => url,
    };

    let urls = String::from_utf8_lossy(res.data())
        .lines()
        .filter(|line| !line.starts_with("#") && !line.trim().is_empty())
        .flat_map(|line| {
            if line.starts_with("http") {
                HttpUrl::parse(line)
            } else {
                HttpUrl::parse(&format!("{}/{}", url_base, line.trim()))
            }
        })
        .collect();

    Ok(urls)
}

fn read_playlist_url_list(file: &str) -> Result<Vec<HttpUrl>, HttpError> {
    let mut file = std::fs::File::open(file)?;
    let mut url_list = Vec::new();
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    for line in buf.lines() {
        if line.starts_with("#") || line.trim().is_empty() {
            continue;
        }
        if line.starts_with("http") {
            let url = HttpUrl::parse(line).map_err(|e| HttpError::InvalidUrl(e.to_string()))?;
            url_list.push(url);
        } else {
            return Err(HttpError::InvalidUrl(format!("Invalid URL in file, cannot locate full resource. Missing protocol scheme. `{line}`")));
        }
    }
    Ok(url_list)
}
fn download_video_list(out_dir: &str, list: &[HttpUrl]) -> Result<(), HttpError> {
    let client = Client::new();
    println!("Downloading videos into `{out_dir}...`");
    for video in cpbar::ProgressBar::new(list.iter()).with_bounds() {
        if let Some(file_name) = video.file() {
            let mut writer = std::fs::File::create(&format!("{}/{}", out_dir, file_name))?;
            client
                .get(&video.to_string())?
                .download_to_file(&mut writer)?;
        } else {
            println!("Could not download file from url: {}", video);
        }
    }

    Ok(())
}

fn consolidate_files(out_dir: &str, name: &str, urls: &[HttpUrl]) -> Result<(), HttpError> {
    println!("Consolidating downloaded stream...");
    let mut output_file = std::fs::File::create(format!("{out_dir}/{name}"))?;

    for video in cpbar::ProgressBar::new(urls.iter()).with_bounds() {
        if let Some(file) = video.file() {
            let mut in_file = std::fs::File::open(format!("{out_dir}/{file}"))?;
            std::io::copy(&mut in_file, &mut output_file)?;
            std::fs::remove_file(format!("{out_dir}/{file}"))?;
        }
    }
    Ok(())
}

fn download_stream(local: bool, url: &str, file: &str) -> Result<(), HttpError> {
    let url_list = if local {
        read_playlist_url_list(url)?
    } else {
        get_playlist_url_list(url)?
    };
    let folder_name = format!("temp_{file}");
    match std::fs::create_dir(&folder_name) {
        Err(e) if e.kind() != std::io::ErrorKind::AlreadyExists => return Err(e.into()),
        _ => (),
    }
    println!("Creating folder: `{folder_name}`");
    download_video_list(&folder_name, &url_list)?;
    consolidate_files(&folder_name, file, &url_list)?;
    Ok(())
}

fn print_help() {
    println!("Download a stream from a url to a m3u8 file.");
    println!("Usage:");
    println!("------");
    println!("download_stream.exe [--url=<url>|--local=<path>] --file=<file name>`");
    println!("------");
    println!("* url:\t\tThe URL to the M3U8 file containing the playlist resources.");
    println!("* local:\tThe path to the local M3U8 file containing the playlist resources.");
    println!("* file:\t\tOutput file name (name of the resource being streamed)");
    println!("\nNote: `--url` and `--local` options are mutually exclusive, if both are passed `--local` takes priority.\n")
}
fn main() -> Result<(), HttpError> {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() < 3 {
        print_help();
        Ok(())
    } else {
        let mut url = None;
        let mut file = None;
        let mut local = false;
        for arg in args {
            if arg.starts_with("--url=") && arg.len() > 6 {
                url = Some(arg.replace("--url=", ""));
            }
            if arg.starts_with("--file=") && arg.len() > 7 {
                file = Some(arg.replace("--file=", ""));
            }
            if arg.starts_with("--local=") && arg.len() > 7 {
                local = true;
                url = Some(arg.replace("--local=", ""));
            }
        }
        if url.is_none() || file.is_none() {
            print_help();
            Ok(())
        } else {
            download_stream(local, &url.unwrap(), &format!("{}.ts", &file.unwrap()))
        }
    }
}
