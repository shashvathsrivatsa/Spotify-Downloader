use crate::utils::to_string_raw;
use crate::models::{PlaylistData, AlbumData, Track, Tracks, TrackItem, TrackInfo, TrackList};

use reqwest::{header, Client};
use serde_json::Value;

use rayon::prelude::*;

pub async fn get_token() -> Result<String, reqwest::Error> {
    let client_id = "a3686b7216004ffca179b98778d8b48d";
    let client_secret = "efa05c5a264a4563a3938c0d70423114";

    let basic_auth = base64::encode(format!("{}:{}", client_id, client_secret));

    let client = Client::new();
    let res = client
        .post("https://accounts.spotify.com/api/token")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(header::AUTHORIZATION, format!("Basic {}", basic_auth))
        .body("grant_type=client_credentials")
        .send()
        .await?;

    let token_data: Value = res.json().await?;
    let access_token = to_string_raw(&token_data["access_token"]);

    Ok(access_token)
}

pub async fn get_track_list(token: String, spotify_link: &str) -> Result<TrackList, reqwest::Error> {

    if spotify_link.contains("playlist") {
        Ok( get_playlist_track_list(token, spotify_link).await? )

    } else if spotify_link.contains("album") {
        Ok( get_album_track_list(token, spotify_link).await? )

    } else if spotify_link.contains("track") {
        Ok( get_song(token, spotify_link).await? )

    } else {
        panic!("Invalid Spotify link")
    }
}

pub async fn get_playlist_track_list(token: String, playlist_link: &str) -> Result<TrackList, reqwest::Error> {
    let playlist_id = playlist_link.split('?').next().unwrap()[34..].to_string();

    let client = Client::new();
    let mut next_url = Some(format!("https://api.spotify.com/v1/playlists/{}", playlist_id));
    let mut track_list_raw: Vec<TrackInfo> = Vec::new();
    let mut playlist_name: Option<String> = None;

    while let Some(url) = next_url {

        let res = client
            .get(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .send().await?;

        let tracks: Vec<TrackItem>;

        if playlist_name.is_none() {
            let playlist_data: PlaylistData = res.json().await?;
            tracks = playlist_data.tracks.items;
            playlist_name = Some(playlist_data.name);
            next_url = playlist_data.tracks.next;
        } else {
            let playlist_data: Tracks = res.json().await?;
            tracks = playlist_data.items;
            next_url = playlist_data.next;
        }

        // TODO: REMOVE ALL CLONES - USE REFERENCES INSTEAD
        track_list_raw.extend(tracks.iter().map(|track_item| TrackInfo {
            title: track_item.track.name.clone(),
            artists: track_item.track.artists.iter().map(|a| a.name.clone()).collect(),
            album_name: track_item.track.album.clone().unwrap().name,
            cover: track_item.track.album.clone().unwrap().images[1].url.clone(),
        }));

    }

    let track_list = TrackList {
        playlist_name: playlist_name.unwrap(),
        tracks: track_list_raw,
    };

    Ok(track_list)
}

pub async fn get_album_track_list(token: String, album_link: &str) -> Result<TrackList, reqwest::Error> {
    let album_id = album_link.split('?').next().unwrap()[31..].to_string();

    let client = Client::new();
    let res = client
        .get(format!("https://api.spotify.com/v1/albums/{}", album_id))
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .await?;

    let playlist_data: AlbumData = res.json().await?;
    let tracks: Vec<Track> = playlist_data.tracks.items;

    let album_name = playlist_data.name;
    let cover = playlist_data.images[0].url.clone();

    let track_list_raw: Vec<TrackInfo> = tracks.par_iter().map(|track_item| TrackInfo {
        title: track_item.name.clone(),
        artists: track_item.artists.par_iter().map(|a| a.name.clone()).collect(),
        album_name: album_name.clone(),
        cover: cover.clone(),
    }).collect();

    let track_list = TrackList {
        playlist_name: album_name,
        tracks: track_list_raw,
    };

    Ok(track_list)
}

pub async fn get_song(token: String, song_link: &str) -> Result<TrackList, reqwest::Error> {
    let song_id = song_link.split('?').next().unwrap()[31..].to_string();

    let client = Client::new();
    let res = client
        .get(format!("https://api.spotify.com/v1/tracks/{}", song_id))
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .await?;

    let track: Track = res.json().await?;

    let album_name = track.album.clone().unwrap().name;

    let track_info = TrackInfo {
        title: track.name,
        artists: track.artists.iter().map(|a| a.name.clone()).collect(),
        album_name: album_name.clone(),
        cover: track.album.unwrap().images[1].url.clone(),
    };

    let track_list = TrackList {
        playlist_name: album_name,
        tracks: vec![track_info],
    };

    Ok(track_list)
}

