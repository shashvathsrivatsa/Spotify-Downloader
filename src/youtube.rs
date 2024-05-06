use std::process::Command;

use reqwest::Client;
use serde_json::Value;

use std::io;

pub fn construct_song_query(title: &str, artist: &str) -> String {
    format!("song {} by {} audio", title, artist)
}

pub async fn get_id(query: &str, search_item: usize) -> Result<String, reqwest::Error> {
    let normalized_query = query.replace(" ", "%20");

    let client = Client::new();
    let res: Value = client
        .get(&format!("https://yt-api.p.rapidapi.com/search?query={}", normalized_query))
        .header("X-RapidAPI-Key", "4d6ef65f19msh9b74fcc7b3ad345p1f9d78jsn4371557c7a02")
        // .header("X-RapidAPI-Key", "08399146damsh0fc2d0852a27fdap107db6jsnd8819fea2a96")
        // .header("X-RapidAPI-Key", "47b2732b42mshbd9c62d7b3775bap15db3bjsndd595eb37c8e")
        // .header("X-RapidAPI-Key", "a39d299db0msh3b80d0d46c4e27fp180e92jsn75d2a6c21553")
        .header("X-RapidAPI-Host", "yt-api.p.rapidapi.com")
        .send().await?
        .json().await?;

    let video_id = res["data"][search_item]["videoId"].as_str().unwrap().to_string();

    Ok(video_id)
}

// TODO: optimize this by combining ffmpegs
pub async fn download_video(
    video_id: &str,
    output_path: &str,
    audio_format: &str,

    artists: &Vec<String>,
    album_name: &str,
    album_cover: &str,
) -> Result<(), io::Error> {

    let video_url = format!("https://www.youtube.com/watch?v={}", video_id);
    let output_path = format!("{}.{}", output_path, audio_format);

    let output = Command::new("yt-dlp")
        .args(&["--extract-audio", "--audio-format", audio_format, "--audio-quality", "0", "-o", &output_path, &video_url])
        .output()?;

    if String::from_utf8_lossy(&output.stderr).contains("Video unavailable") {
        return Err(io::Error::new(io::ErrorKind::Other, "Video unavailable"));
    } else if !output.status.success() {
        println!("{:?}", output);
        return Err(io::Error::new(io::ErrorKind::Other, "Download failed"));
    }

    if audio_format == "wav" {
        let temp_new_path = format!("{}_temp.wav", output_path);
        Command::new("ffmpeg")
            .args(&["-i", &output_path, "-c:a", "pcm_s32le", &temp_new_path])
            .output()?;

        std::fs::rename(&temp_new_path, &output_path)?;
    } else if audio_format == "mp3" {
        let album_cover_path = format!("/tmp/{}_album_cover.jpg", video_id);
        let response = reqwest::get(album_cover).await.unwrap();
        let bytes = response.bytes().await.unwrap();
        std::fs::write(&album_cover_path, &bytes)?;

        let temp_output_path = format!("{}_temp.{}", output_path, audio_format);
        Command::new("ffmpeg")
            .args(&[
                "-i", &output_path,
                "-i", &album_cover_path,
                "-map", "0:0", "-map", "1:0", "-c", "copy", "-id3v2_version", "3",
                "-metadata", &format!("artist={}", artists.join(", ")),
                "-metadata", &format!("album={}", album_name),
                &temp_output_path,
            ])
            .output()?;

        std::fs::rename(&temp_output_path, &output_path)?;

        std::fs::remove_file(album_cover_path)?;
    }

    Ok(())
}
