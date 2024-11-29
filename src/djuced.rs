// Copyright (c) 2024 Jake Walker
// 
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use anyhow::{anyhow, Context, Result};
use directories::UserDirs;
use sqlite::{Connection, OpenFlags, Value};

use crate::{HotCue, Song};

pub fn djuced_db_path() -> Result<String> {
    Ok(UserDirs::new()
        .context("failed to build user dir")?
        .document_dir()
        .context("failed to build documents dir")?
        .join("DJUCED/DJUCED.db")
        .to_str()
        .context("failed to build djuced path")?
        .to_owned())
}

pub fn djuced_db_connection() -> Result<Connection> {
    Connection::open_with_flags(djuced_db_path()?, OpenFlags::new().with_read_write())
        .context("failed to open database")
}

pub fn convert_key(camelot_key: &str) -> Result<i64> {
    match camelot_key {
        "1A" => Ok(23),  // 6m
        "1B" => Ok(2),   // 6D
        "2A" => Ok(18),  // 7m
        "2B" => Ok(9),   // 7D
        "3A" => Ok(13),  // 8m
        "3B" => Ok(4),   // 8D
        "4A" => Ok(20),  // 9m
        "4B" => Ok(11),  // 9D
        "5A" => Ok(15),  // 10
        "5B" => Ok(6),   // 10D
        "6A" => Ok(22),  // 11m
        "6B" => Ok(1),   // 11D
        "7A" => Ok(17),  // 12m
        "7B" => Ok(8),   // 12D
        "8A" => Ok(12),  // 1m
        "8B" => Ok(3),   // 1D
        "9A" => Ok(19),  // 2m
        "9B" => Ok(10),  // 2D
        "10A" => Ok(14), // 3m
        "10B" => Ok(5),  // 3D
        "11A" => Ok(21), // 4m
        "11B" => Ok(0),  // 4D
        "12A" => Ok(16), // 5m
        "12B" => Ok(7),  // 5D
        _ => Err(anyhow!("invalid camelot key {}", camelot_key)),
    }
}

pub fn update_cues(conn: &Connection, track_id: &str, cues: Vec<HotCue>) -> Result<()> {
    let mut stmt = conn
        .prepare("DELETE FROM trackCues WHERE trackId = ? AND cuenumber < 1000")
        .context("failed to create delete prepared statement")?;
    stmt.bind((1, track_id))
        .context("failed to bind track id to delete statement")?;
    stmt.next().context("failed to execute delete statement")?;

    let mut cues = cues;
    cues.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());

    for (i, cue) in cues.iter().enumerate() {
        let mut stmt = conn.prepare("INSERT INTO trackCues (trackId, cuename, cuenumber, cuepos, loopLength, cueColor, isSavedLoop) VALUES (:path, :name, :number, :pos, 0, :color, 0)").context("failed to create add cue statment")?;
        stmt.bind_iter::<_, (_, Value)>([
            (":path", track_id.into()),
            (":name", format!("Cue {}", i).into()),
            (":number", (i as i64).into()),
            (":pos", cue.time.into()),
            (":color", (i as i64).into()),
        ])
        .context("failed to bind cue statement")?;
        stmt.next().context("failed to execute cue statement")?;
    }

    Ok(())
}

pub fn update_song(conn: &Connection, track_id: &str, song: &Song) -> Result<()> {
    let mut stmt = conn.prepare("UPDATE tracks SET artist = :artist, title = :title, bpm = :bpm, key = :key, comment = :comment WHERE absolutepath = :path").context("failed to create update statement")?;
    stmt.bind_iter::<_, (_, Value)>([
        (":artist", song.artist.clone().into()),
        (":title", song.name.clone().into()),
        (":bpm", song.tempo.into()),
        (":key", convert_key(&song.key)?.into()),
        (":path", track_id.into()),
        (
            ":comment",
            format!("(*) {} - Energy {}", song.key, song.energy).into(),
        ),
    ])
    .context("failed to bind update statement")?;
    stmt.next().context("failed to execute update statement")?;

    Ok(())
}
