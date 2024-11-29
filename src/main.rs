// Copyright (c) 2024 Jake Walker
// 
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use anyhow::{Context, Result};
use djuced::{djuced_db_connection, djuced_db_path, update_cues, update_song};
use mik::{get_analysed_songs, mik_db_connection, parse_bookmark_data};

mod djuced;
mod mik;

#[derive(Debug, Clone)]
pub struct HotCue {
    time: f64,
    name: Option<String>,
}

#[derive(Debug)]
pub struct Song {
    mik_id: i64,
    file_size: i64,
    energy: f64,
    tempo: f64,
    key: String,
    artist: String,
    name: String,
    path: String,
    hot_cues: Vec<HotCue>,
}

fn main() -> Result<()> {
    println!("{:?}", djuced_db_path());

    let mik_conn = mik_db_connection().context("failed to open mik database")?;
    let djuced_conn = djuced_db_connection().context("failed to open djuced database")?;

    let songs = get_analysed_songs(&mik_conn)?;

    // let s = songs.first().unwrap();

    // println!("{:#?}", s);

    // update_song(&djuced_conn, &s.path, s)?;

    // update_cues(&djuced_conn, &s.path, s.hot_cues.clone())?;

    // let ids = find_track_ids(&djuced_conn, &s.name, &s.artist)?;

    // println!("{:?}", ids);

    for song in songs {
        println!("updating {}...", song.path);
        update_song(&djuced_conn, &song.path, &song)?;
        update_cues(&djuced_conn, &song.path, song.hot_cues.clone())?;
    }

    Ok(())
}
