mod utils; use utils::{clean_old_playlist, normalize_string};
mod models; use models::TrackList;
mod spotify; use spotify::{get_token, get_track_list};
mod youtube; use youtube::{construct_song_query, get_id, download_video};

use std::io;
use std::env;
use std::time::Instant;

use num_cpus;
use tokio::task;
use futures::future::join_all;

async fn download_playlist(track_list: TrackList, audio_format: String, container: String) {
    let start = Instant::now();

    let num_tracks = track_list.tracks.len();
    let num_cpus = num_cpus::get();
    let num_chunks = (num_tracks + num_cpus - 1) / num_cpus;

    let mut tasks = Vec::with_capacity(num_chunks);

    for chunk in track_list.tracks.chunks(num_chunks) {
        let cloned_chunk = chunk.to_vec();
        let cloned_track_list = track_list.clone();
        let audio_format = audio_format.clone();
        let container = container.clone();

        let task = task::spawn(async move {
            for track in cloned_chunk {
                let song_query = construct_song_query(&track.title, &track.artists[0]);
                let output_path = format!("{}/{}/{}", container, normalize_string(&cloned_track_list.playlist_name), normalize_string(&track.title));

                for i in 0.. {
                    let video_id = match get_id(&song_query, i).await {
                        Ok(id) => id,
                        Err(e) => panic!("Error getting id: {}", e),
                    };

                    match download_video(
                        &video_id,
                        &output_path,
                        &audio_format,
                        &track.artists,
                        &track.album_name,
                        &track.cover,
                    ).await {
                        Ok(_) => break,
                        Err(e) => if e.to_string() != "Video unavailable" { panic!("Error downloading video: {}", e) }
                    };
                }

                println!("Downloaded: {}", track.title);
            }
        });

        tasks.push(task);
    }

    join_all(tasks).await;

    let duration = start.elapsed();
    println!("Downloaded {} songs in {:?}", track_list.tracks.len(), duration);
}

#[tokio::main]
async fn main() {
    let start = Instant::now();

    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        panic!("Usage: {} <playlist_link> <audio_format> <container directory for playlist>", args[0]);
    }
    let spotify_link: &str = &args[1];
    let audio_format: String = args[2].clone();
    let container: String = args[3].clone();

    let token = match get_token().await {
        Ok(token) => token,
        Err(e) => panic!("Error getting token: {}", e),
    };

    let track_list: TrackList = match get_track_list(token, spotify_link).await {
        Ok(track_list) => track_list,
        Err(e) => panic!("Error getting track list: {}", e),
    };

    match clean_old_playlist(format!("{}/{}", container, normalize_string(&track_list.playlist_name)).as_str()) {
        Ok(_) => (),
        Err(e) => {
            if e.kind() != io::ErrorKind::NotFound {
                panic!("Error cleaning old playlist: {}", e);
            }
        },
    }

    let duration = start.elapsed();
    println!("Retrieved track list in {:?}", duration);

    download_playlist(track_list, audio_format, container).await;

}

