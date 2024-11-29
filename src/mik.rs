// Copyright (c) 2024 Jake Walker
// 
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use core::str;
use std::{env, path::Path};

use anyhow::{anyhow, Context, Result};
use directories::ProjectDirs;
use itertools::{self, Itertools};
use sqlite::{Connection, OpenFlags};

use crate::{HotCue, Song};

const ROUND_THRESHOLD: f64 = 0.03;

pub fn mik_path() -> Result<String> {
    match env::consts::OS {
        "macos" => Ok(ProjectDirs::from_path("Mixedinkey".into())
            .context("could not build project path")?
            .config_local_dir()
            .to_str()
            .context("could not build config local path")?
            .to_string()),
        _ => Err(anyhow!("unsupported os {}", env::consts::OS)),
    }
}

pub fn mik_db_filename() -> Result<String> {
    match env::consts::OS {
        "macos" => Ok("Collection11.mikdb".into()),
        _ => Err(anyhow!("unsupported os {}", env::consts::OS)),
    }
}

pub fn mik_db_connection() -> Result<Connection> {
    let db_path = Path::new(&mik_path()?).join(&mik_db_filename()?);
    Connection::open_with_flags(db_path, OpenFlags::new().with_read_only())
        .context("failed to open database")
}

pub fn get_song_cues(conn: &Connection, song_id: i64) -> Result<Vec<HotCue>> {
    let query = "SELECT ZTIME, ZNAME FROM ZCUEPOINT WHERE ZSONG = ?";

    let mut cues = Vec::new();

    for row in conn
        .prepare(query)?
        .into_iter()
        .bind((1, song_id))?
        .map(|c| c.unwrap())
    {
        cues.push(HotCue {
            time: row.read::<f64, _>("ZTIME"),
            name: row
                .read::<Option<&str>, _>("ZNAME")
                .and_then(|s| Some(s.to_string())),
        });
    }

    Ok(cues)
}

pub fn parse_bookmark_data(bookmark_data: &[u8]) -> Vec<String> {
    let data_markers = bookmark_data
        .windows(8)
        .enumerate()
        .filter(|(_, x)| x[1..] == [0x00, 0x00, 0x00, 0x01, 0x01, 0x00, 0x00])
        .map(|(i, x)| (i, x[0] as usize))
        .collect::<Vec<(usize, usize)>>();

    data_markers
        .into_iter()
        .map(|(index, length)| {
            let start = index + 8;
            let end = start + length;
            String::from_utf8_lossy(&bookmark_data[start..end]).into_owned()
        })
        .collect::<Vec<String>>()
}

pub fn round_tempo(t: f64) -> f64 {
    let rounded = t.round();
    if (t - rounded).abs() < ROUND_THRESHOLD {
        rounded
    } else {
        t
    }
}

pub fn get_analysed_songs(conn: &Connection) -> Result<Vec<Song>> {
    let query =
        "SELECT Z_PK, ZFILESIZE, ZENERGY, ZTEMPO, ZKEY, ZARTIST, ZNAME, ZBOOKMARKDATA FROM ZSONG";

    let mut songs = Vec::new();

    for row in conn.prepare(query)?.into_iter().map(|s| s.unwrap()) {
        let id = row.read::<i64, _>("Z_PK");
        let bookmark_data = row.read::<&[u8], _>("ZBOOKMARKDATA");

        songs.push(Song {
            mik_id: id,
            file_size: row.read::<i64, _>("ZFILESIZE"),
            energy: row.read::<f64, _>("ZENERGY"),
            tempo: round_tempo(row.read::<f64, _>("ZTEMPO")),
            key: row.read::<&str, _>("ZKEY").into(),
            artist: row.read::<&str, _>("ZARTIST").into(),
            name: row.read::<&str, _>("ZNAME").into(),
            hot_cues: get_song_cues(conn, id)?,
            // assume the last part of the bookmark data is the path
            path: parse_bookmark_data(bookmark_data)
                .last()
                .context("bookmark data is empty")?
                .to_string(),
        });
    }

    Ok(songs)
}
