use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct PlaylistData {
    pub tracks: Tracks,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Tracks {
    pub items: Vec<TrackItem>,
    pub next: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TrackItem {
    pub track: Track,
}

#[derive(Debug, Deserialize)]
pub struct Track {
    pub name: String,
    pub artists: Vec<Artist>,
    pub album: Option<Album>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Album {
    pub name: String,
    pub images: Vec<Image>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Image {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Artist {
    pub name: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct TrackInfo {
    pub title: String,
    pub artists: Vec<String>,
    pub album_name: String,
    pub cover: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct TrackList {
    pub playlist_name: String,
    pub tracks: Vec<TrackInfo>,
}

#[derive(Debug, Deserialize)]
pub struct AlbumData {
    pub tracks: AlbumTracks,
    pub name: String,
    pub images: Vec<Image>,
}

#[derive(Debug, Deserialize)]
pub struct AlbumTracks {
    pub items: Vec<Track>,
    pub next: Option<String>,
}

