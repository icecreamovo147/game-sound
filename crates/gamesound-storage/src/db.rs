use anyhow::{bail, Result};
use chrono::Utc;
use gamesound_core::{hotkey::is_reserved_tui_hotkey, Category, PlaybackMode, Sound};
use rusqlite::{params, Connection, OptionalExtension};
use std::{
    fs,
    path::{Path, PathBuf},
};
#[derive(Debug, Clone)]
pub struct Profile {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub is_active: bool,
}
#[derive(serde::Serialize, serde::Deserialize)]
struct ProfileExport {
    version: u8,
    sounds: Vec<Sound>,
    categories: Vec<Category>,
    #[serde(default)]
    hotkeys: Vec<ExportHotkey>,
}
#[derive(serde::Serialize, serde::Deserialize)]
struct ExportHotkey {
    sound_id: i64,
    hotkey: String,
}
pub struct Library {
    conn: Connection,
}
impl Library {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        let this = Self { conn };
        this.migrate()?;
        this.ensure_default_profile()?;
        Ok(this)
    }
    pub fn schema_version(&self) -> Result<i64> {
        Ok(self.conn.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )?)
    }
    fn migrate(&self) -> Result<()> {
        self.conn.execute_batch("CREATE TABLE IF NOT EXISTS schema_migrations(version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL);")?;
        let applied: Option<i64> = self
            .conn
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .optional()?
            .flatten();
        if applied.unwrap_or(0) < 1 {
            let transaction = self.conn.unchecked_transaction()?;
            transaction.execute_batch("CREATE TABLE IF NOT EXISTS profiles(id INTEGER PRIMARY KEY AUTOINCREMENT,name TEXT NOT NULL UNIQUE,description TEXT NOT NULL DEFAULT '',is_active INTEGER NOT NULL DEFAULT 0,created_at TEXT NOT NULL,updated_at TEXT NOT NULL); CREATE TABLE IF NOT EXISTS categories(id INTEGER PRIMARY KEY AUTOINCREMENT,name TEXT NOT NULL,profile_id INTEGER,sort_order INTEGER NOT NULL DEFAULT 0,created_at TEXT NOT NULL,updated_at TEXT NOT NULL); CREATE TABLE IF NOT EXISTS sounds(id INTEGER PRIMARY KEY AUTOINCREMENT,name TEXT NOT NULL,file_path TEXT NOT NULL,category_id INTEGER,profile_id INTEGER,volume REAL NOT NULL DEFAULT .8,playback_mode TEXT NOT NULL DEFAULT 'overlay',loop_enabled INTEGER NOT NULL DEFAULT 0,favorite INTEGER NOT NULL DEFAULT 0,tags TEXT NOT NULL DEFAULT '',note TEXT NOT NULL DEFAULT '',sort_order INTEGER NOT NULL DEFAULT 0,play_count INTEGER NOT NULL DEFAULT 0,last_played_at TEXT,created_at TEXT NOT NULL,updated_at TEXT NOT NULL); CREATE TABLE IF NOT EXISTS hotkeys(id INTEGER PRIMARY KEY AUTOINCREMENT,profile_id INTEGER,sound_id INTEGER,action TEXT NOT NULL,hotkey TEXT NOT NULL UNIQUE,enabled INTEGER NOT NULL DEFAULT 1,created_at TEXT NOT NULL,updated_at TEXT NOT NULL); CREATE TABLE IF NOT EXISTS playback_history(id INTEGER PRIMARY KEY AUTOINCREMENT,sound_id INTEGER NOT NULL,triggered_by TEXT NOT NULL,played_at TEXT NOT NULL);")?;
            transaction.execute(
                "INSERT INTO schema_migrations(version,applied_at) VALUES(1,?1)",
                [Utc::now().to_rfc3339()],
            )?;
            transaction.commit()?;
        }
        if applied.unwrap_or(0) < 2 {
            let transaction = self.conn.unchecked_transaction()?;
            transaction.execute_batch("ALTER TABLE sounds ADD COLUMN duration_ms INTEGER; ALTER TABLE sounds ADD COLUMN sample_rate INTEGER; ALTER TABLE sounds ADD COLUMN channels INTEGER; ALTER TABLE sounds ADD COLUMN file_size INTEGER; CREATE TABLE IF NOT EXISTS app_settings(key TEXT PRIMARY KEY,value TEXT NOT NULL,updated_at TEXT NOT NULL);")?;
            transaction.execute(
                "INSERT INTO schema_migrations(version,applied_at) VALUES(2,?1)",
                [Utc::now().to_rfc3339()],
            )?;
            transaction.commit()?;
        }
        Ok(())
    }
    fn ensure_default_profile(&self) -> Result<()> {
        if self
            .conn
            .query_row("SELECT 1 FROM profiles LIMIT 1", [], |r| r.get::<_, i32>(0))
            .optional()?
            .is_none()
        {
            let n = Utc::now().to_rfc3339();
            self.conn.execute("INSERT INTO profiles(name,is_active,created_at,updated_at) VALUES('default',1,?1,?1)",[n])?;
        }
        Ok(())
    }
    pub fn add_sound(&self, mut sound: Sound) -> Result<i64> {
        Self::validate_audio_path(&sound.file_path)?;
        if !std::path::Path::new(&sound.file_path).is_file() {
            bail!("audio file does not exist: {}", sound.file_path)
        }
        let now = Utc::now().to_rfc3339();
        let metadata = std::fs::metadata(&sound.file_path).ok();
        let audio_metadata = gamesound_core::audio::probe_file(Path::new(&sound.file_path)).ok();
        let (sample_rate, channels, duration_ms) = audio_metadata
            .map(|(rate, channels, samples)| {
                let duration =
                    samples.saturating_mul(1_000) / rate.max(1) as u64 / channels.max(1) as u64;
                (
                    Some(rate as i64),
                    Some(channels as i64),
                    Some(duration as i64),
                )
            })
            .unwrap_or((None, None, None));
        let file_size = metadata.map(|m| m.len() as i64);
        self.conn.execute("INSERT INTO sounds(name,file_path,category_id,profile_id,volume,playback_mode,loop_enabled,favorite,tags,note,sort_order,duration_ms,sample_rate,channels,file_size,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?16)",params![sound.name,sound.file_path,sound.category_id,sound.profile_id,sound.volume,sound.playback_mode.as_str(),sound.loop_enabled,sound.favorite,sound.tags,sound.note,sound.sort_order,duration_ms,sample_rate,channels,file_size,now])?;
        sound.id = self.conn.last_insert_rowid();
        Ok(sound.id)
    }
    /// Imports one audio file or scans a directory recursively for supported audio files.
    pub fn import_path(&self, path: &Path, profile_id: Option<i64>) -> Result<Vec<i64>> {
        self.import_path_with_mode(path, profile_id, None)
    }
    /// Imports by reference, or copies source files into a managed directory when requested.
    pub fn import_path_with_mode(
        &self,
        path: &Path,
        profile_id: Option<i64>,
        copy_to: Option<&Path>,
    ) -> Result<Vec<i64>> {
        let mut files = Vec::new();
        collect_audio_files(path, &mut files)?;
        if files.is_empty() {
            bail!("no supported audio files found at {}", path.display());
        }
        if let Some(directory) = copy_to {
            fs::create_dir_all(directory)?;
        }
        files
            .into_iter()
            .map(|source| {
                let path = if let Some(directory) = copy_to {
                    copy_sound_file(&source, directory)?
                } else {
                    source
                };
                let name = path
                    .file_stem()
                    .and_then(|name| name.to_str())
                    .unwrap_or("Sound")
                    .to_owned();
                self.add_sound(Sound {
                    id: 0,
                    name,
                    file_path: path.to_string_lossy().into_owned(),
                    category_id: None,
                    profile_id,
                    volume: 0.8,
                    playback_mode: PlaybackMode::Overlay,
                    loop_enabled: false,
                    favorite: false,
                    tags: String::new(),
                    note: String::new(),
                    sort_order: 0,
                    play_count: 0,
                    last_played_at: None,
                })
            })
            .collect()
    }
    pub fn add_category(&self, name: &str, profile_id: Option<i64>) -> Result<i64> {
        let name = name.trim();
        if name.is_empty() {
            bail!("category name cannot be empty");
        }
        let now = Utc::now().to_rfc3339();
        self.conn.execute("INSERT INTO categories(name,profile_id,sort_order,created_at,updated_at) VALUES(?1,?2,(SELECT COALESCE(MAX(sort_order),-1)+1 FROM categories),?3,?3)", params![name, profile_id, now])?;
        Ok(self.conn.last_insert_rowid())
    }
    pub fn rename_category(&self, id: i64, name: &str) -> Result<()> {
        let name = name.trim();
        if name.is_empty() {
            bail!("category name cannot be empty");
        }
        self.conn.execute(
            "UPDATE categories SET name=?1,updated_at=?2 WHERE id=?3",
            params![name, Utc::now().to_rfc3339(), id],
        )?;
        Ok(())
    }
    /// Removes only the category. Its sounds safely return to the All Sounds list.
    pub fn remove_category(&self, id: i64) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            "UPDATE sounds SET category_id=NULL WHERE category_id=?1",
            [id],
        )?;
        tx.execute("DELETE FROM categories WHERE id=?1", [id])?;
        tx.commit()?;
        Ok(())
    }
    pub fn profiles(&self) -> Result<Vec<Profile>> {
        let mut statement = self
            .conn
            .prepare("SELECT id,name,description,is_active FROM profiles ORDER BY name")?;
        let profiles = statement
            .query_map([], |row| {
                Ok(Profile {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    is_active: row.get(3)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(profiles)
    }
    pub fn add_profile(&self, name: &str, description: &str) -> Result<i64> {
        let name = name.trim();
        if name.is_empty() {
            bail!("profile name cannot be empty");
        }
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO profiles(name,description,created_at,updated_at) VALUES(?1,?2,?3,?3)",
            params![name, description, now],
        )?;
        Ok(self.conn.last_insert_rowid())
    }
    pub fn set_active_profile(&self, id: i64) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute("UPDATE profiles SET is_active=0", [])?;
        if tx.execute(
            "UPDATE profiles SET is_active=1,updated_at=?1 WHERE id=?2",
            params![Utc::now().to_rfc3339(), id],
        )? != 1
        {
            bail!("profile does not exist");
        }
        tx.commit()?;
        Ok(())
    }
    pub fn active_profile(&self) -> Result<Profile> {
        self.conn
            .query_row(
                "SELECT id,name,description,is_active FROM profiles WHERE is_active=1 LIMIT 1",
                [],
                |row| {
                    Ok(Profile {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        description: row.get(2)?,
                        is_active: row.get(3)?,
                    })
                },
            )
            .map_err(Into::into)
    }
    pub fn update_sound(&self, s: &Sound) -> Result<()> {
        let n = Utc::now().to_rfc3339();
        self.conn.execute("UPDATE sounds SET name=?1,category_id=?2,volume=?3,playback_mode=?4,loop_enabled=?5,favorite=?6,tags=?7,note=?8,sort_order=?9,updated_at=?10 WHERE id=?11",params![s.name,s.category_id,s.volume,s.playback_mode.as_str(),s.loop_enabled,s.favorite,s.tags,s.note,s.sort_order,n,s.id])?;
        Ok(())
    }
    pub fn remove_sound(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM hotkeys WHERE sound_id=?1", [id])?;
        self.conn.execute("DELETE FROM sounds WHERE id=?1", [id])?;
        Ok(())
    }
    pub fn record_play(&self, id: i64, triggered_by: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE sounds SET play_count=play_count+1,last_played_at=?1,updated_at=?1 WHERE id=?2",
            params![now, id],
        )?;
        self.conn.execute(
            "INSERT INTO playback_history(sound_id,triggered_by,played_at) VALUES(?1,?2,?3)",
            params![id, triggered_by, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }
    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO app_settings(key,value,updated_at) VALUES(?1,?2,?3) ON CONFLICT(key) DO UPDATE SET value=excluded.value,updated_at=excluded.updated_at",
            params![key, value, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }
    pub fn setting(&self, key: &str) -> Result<Option<String>> {
        self.conn
            .query_row(
                "SELECT value FROM app_settings WHERE key=?1",
                [key],
                |row| row.get(0),
            )
            .optional()
            .map_err(Into::into)
    }
    pub fn sounds(&self, category: Option<i64>, query: &str) -> Result<Vec<Sound>> {
        let mut st=self.conn.prepare("SELECT id,name,file_path,category_id,profile_id,volume,playback_mode,loop_enabled,favorite,tags,note,sort_order,play_count,last_played_at FROM sounds WHERE (?1 IS NULL OR category_id=?1) AND (name LIKE '%'||?2||'%' OR tags LIKE '%'||?2||'%') ORDER BY sort_order,name")?;
        let sounds = st
            .query_map(params![category, query], row_sound)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into);
        sounds
    }
    pub fn sounds_in_profile(
        &self,
        profile_id: i64,
        category: Option<i64>,
        query: &str,
    ) -> Result<Vec<Sound>> {
        let mut statement = self.conn.prepare("SELECT id,name,file_path,category_id,profile_id,volume,playback_mode,loop_enabled,favorite,tags,note,sort_order,play_count,last_played_at FROM sounds WHERE profile_id=?1 AND (?2 IS NULL OR category_id=?2) AND (name LIKE '%'||?3||'%' OR tags LIKE '%'||?3||'%') ORDER BY sort_order,name")?;
        let sounds = statement
            .query_map(params![profile_id, category, query], row_sound)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(sounds)
    }
    pub fn favorite_sounds_in_profile(&self, profile_id: i64, query: &str) -> Result<Vec<Sound>> {
        self.sounds_by_clause(profile_id, query, "favorite=1", "sort_order,name")
    }
    pub fn recent_sounds_in_profile(&self, profile_id: i64, query: &str) -> Result<Vec<Sound>> {
        self.sounds_by_clause(
            profile_id,
            query,
            "play_count>0",
            "last_played_at DESC, name",
        )
    }
    fn sounds_by_clause(
        &self,
        profile_id: i64,
        query: &str,
        clause: &str,
        order: &str,
    ) -> Result<Vec<Sound>> {
        let sql = format!("SELECT id,name,file_path,category_id,profile_id,volume,playback_mode,loop_enabled,favorite,tags,note,sort_order,play_count,last_played_at FROM sounds WHERE profile_id=?1 AND {clause} AND (name LIKE '%'||?2||'%' OR tags LIKE '%'||?2||'%') ORDER BY {order}");
        let mut statement = self.conn.prepare(&sql)?;
        let sounds = statement
            .query_map(params![profile_id, query], row_sound)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(sounds)
    }
    pub fn categories(&self) -> Result<Vec<Category>> {
        let mut st = self.conn.prepare(
            "SELECT id,name,profile_id,sort_order FROM categories ORDER BY sort_order,name",
        )?;
        let categories = st
            .query_map([], |r| {
                Ok(Category {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    profile_id: r.get(2)?,
                    sort_order: r.get(3)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into);
        categories
    }
    pub fn categories_in_profile(&self, profile_id: i64) -> Result<Vec<Category>> {
        self.categories_for_profile(Some(profile_id))
    }
    pub fn set_hotkey(&self, sound_id: i64, hotkey: &str) -> Result<()> {
        if is_reserved_tui_hotkey(hotkey) {
            bail!("'{hotkey}' is reserved for TUI controls; use Ctrl, Command, Alt, or Shift with the key");
        }
        let conflict: Option<i64> = self
            .conn
            .query_row(
                "SELECT sound_id FROM hotkeys WHERE hotkey=?1",
                [hotkey],
                |r| r.get(0),
            )
            .optional()?;
        if let Some(id) = conflict {
            if id != sound_id {
                bail!("hotkey is already assigned to sound {id}")
            }
        }
        let n = Utc::now().to_rfc3339();
        self.conn
            .execute("DELETE FROM hotkeys WHERE sound_id=?1", [sound_id])?;
        self.conn.execute("INSERT INTO hotkeys(sound_id,action,hotkey,created_at,updated_at) VALUES(?1,'play',?2,?3,?3)",params![sound_id,hotkey,n])?;
        Ok(())
    }
    pub fn hotkey(&self, id: i64) -> Result<Option<String>> {
        self.conn
            .query_row("SELECT hotkey FROM hotkeys WHERE sound_id=?1", [id], |r| {
                r.get(0)
            })
            .optional()
            .map_err(Into::into)
    }
    pub fn clear_hotkey(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM hotkeys WHERE sound_id=?1", [id])?;
        Ok(())
    }
    /// Cleans up bindings created by older versions before TUI-key protection
    /// existed. Returns the number of removed invalid bindings.
    pub fn clear_reserved_tui_hotkeys(&self) -> Result<usize> {
        let mut statement = self.conn.prepare("SELECT id,hotkey FROM hotkeys")?;
        let invalid = statement
            .query_map([], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?
            .into_iter()
            .filter(|(_, hotkey)| is_reserved_tui_hotkey(hotkey))
            .collect::<Vec<_>>();
        for (id, _) in &invalid {
            self.conn.execute("DELETE FROM hotkeys WHERE id=?1", [id])?;
        }
        Ok(invalid.len())
    }
    pub fn export_profile_json(&self, profile_id: Option<i64>) -> Result<String> {
        let sounds = self.sounds_for_profile(profile_id)?;
        let categories = self.categories_for_profile(profile_id)?;
        let hotkeys = self.hotkeys_for_sounds(&sounds)?;
        Ok(serde_json::to_string_pretty(&ProfileExport {
            version: 1,
            sounds,
            categories,
            hotkeys,
        })?)
    }
    /// Imports a manifest without trusting its database IDs. Categories are remapped
    /// and missing source files remain visible in the library as invalid entries.
    pub fn import_profile_json(&self, input: &str, profile_id: Option<i64>) -> Result<usize> {
        let exported: ProfileExport = serde_json::from_str(input)?;
        if exported.version != 1 {
            bail!("unsupported GameSound export version: {}", exported.version);
        }
        let mut category_ids = std::collections::HashMap::new();
        for category in exported.categories {
            let new_id = self.add_category(&category.name, profile_id)?;
            category_ids.insert(category.id, new_id);
        }
        let now = Utc::now().to_rfc3339();
        let transaction = self.conn.unchecked_transaction()?;
        let mut count = 0;
        let mut sound_ids = std::collections::HashMap::new();
        for sound in exported.sounds {
            transaction.execute("INSERT INTO sounds(name,file_path,category_id,profile_id,volume,playback_mode,loop_enabled,favorite,tags,note,sort_order,play_count,last_played_at,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?14)", params![sound.name,sound.file_path,sound.category_id.and_then(|id|category_ids.get(&id).copied()),profile_id,sound.volume,sound.playback_mode.as_str(),sound.loop_enabled,sound.favorite,sound.tags,sound.note,sound.sort_order,sound.play_count,sound.last_played_at,now])?;
            sound_ids.insert(sound.id, transaction.last_insert_rowid());
            count += 1;
        }
        for hotkey in exported.hotkeys {
            if let Some(sound_id) = sound_ids.get(&hotkey.sound_id) {
                transaction.execute("INSERT OR IGNORE INTO hotkeys(profile_id,sound_id,action,hotkey,enabled,created_at,updated_at) VALUES(?1,?2,'play',?3,1,?4,?4)", params![profile_id, sound_id, hotkey.hotkey, now])?;
            }
        }
        transaction.commit()?;
        Ok(count)
    }
    fn sounds_for_profile(&self, profile_id: Option<i64>) -> Result<Vec<Sound>> {
        let mut st=self.conn.prepare("SELECT id,name,file_path,category_id,profile_id,volume,playback_mode,loop_enabled,favorite,tags,note,sort_order,play_count,last_played_at FROM sounds WHERE (?1 IS NULL OR profile_id=?1) ORDER BY sort_order,name")?;
        let sounds = st
            .query_map([profile_id], row_sound)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(sounds)
    }
    fn categories_for_profile(&self, profile_id: Option<i64>) -> Result<Vec<Category>> {
        let mut st=self.conn.prepare("SELECT id,name,profile_id,sort_order FROM categories WHERE (?1 IS NULL OR profile_id=?1) ORDER BY sort_order,name")?;
        let categories = st
            .query_map([profile_id], |r| {
                Ok(Category {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    profile_id: r.get(2)?,
                    sort_order: r.get(3)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(categories)
    }
    fn hotkeys_for_sounds(&self, sounds: &[Sound]) -> Result<Vec<ExportHotkey>> {
        let mut hotkeys = Vec::new();
        for sound in sounds {
            if let Some(hotkey) = self.hotkey(sound.id)? {
                hotkeys.push(ExportHotkey {
                    sound_id: sound.id,
                    hotkey,
                });
            }
        }
        Ok(hotkeys)
    }
    fn validate_audio_path(path: &str) -> Result<()> {
        let path = Path::new(path);
        if !path.is_file() {
            bail!("audio file does not exist: {}", path.display());
        }
        const EXTENSIONS: &[&str] = &["wav", "mp3", "m4a", "aac", "ogg", "flac"];
        let extension = path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        if !EXTENSIONS.contains(&extension.as_str()) {
            bail!(
                "unsupported audio format: .{extension}; supported: WAV, MP3, M4A, AAC, OGG, FLAC"
            );
        }
        Ok(())
    }
}
fn copy_sound_file(source: &Path, directory: &Path) -> Result<PathBuf> {
    let filename = source
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("audio path has no filename: {}", source.display()))?;
    let mut candidate = directory.join(filename);
    let stem = source
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("sound");
    let extension = source
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("audio");
    let mut number = 1;
    while candidate.exists() {
        candidate = directory.join(format!("{stem}-{number}.{extension}"));
        number += 1;
    }
    fs::copy(source, &candidate)?;
    Ok(candidate)
}
fn collect_audio_files(path: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    if path.is_file() {
        Library::validate_audio_path(&path.to_string_lossy())?;
        files.push(path.to_owned());
        return Ok(());
    }
    if !path.is_dir() {
        bail!("path does not exist: {}", path.display());
    }
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        if entry_path.is_dir() {
            collect_audio_files(&entry_path, files)?;
        } else if entry_path.is_file()
            && entry_path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| {
                    matches!(
                        e.to_ascii_lowercase().as_str(),
                        "wav" | "mp3" | "m4a" | "aac" | "ogg" | "flac"
                    )
                })
                .unwrap_or(false)
        {
            files.push(entry_path);
        }
    }
    Ok(())
}
fn row_sound(r: &rusqlite::Row<'_>) -> rusqlite::Result<Sound> {
    let mode: String = r.get(6)?;
    Ok(Sound {
        id: r.get(0)?,
        name: r.get(1)?,
        file_path: r.get(2)?,
        category_id: r.get(3)?,
        profile_id: r.get(4)?,
        volume: r.get(5)?,
        playback_mode: match mode.as_str() {
            "interrupt" => PlaybackMode::Interrupt,
            "queue" => PlaybackMode::Queue,
            "exclusive" => PlaybackMode::Exclusive,
            _ => PlaybackMode::Overlay,
        },
        loop_enabled: r.get(7)?,
        favorite: r.get(8)?,
        tags: r.get(9)?,
        note: r.get(10)?,
        sort_order: r.get(11)?,
        play_count: r.get(12)?,
        last_played_at: r.get(13)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("gamesound-storage-{name}-{}", std::process::id()))
    }

    #[test]
    fn categories_preserve_sounds_when_deleted() {
        let root = temp_path("categories");
        fs::create_dir_all(&root).unwrap();
        let audio = root.join("effect.wav");
        fs::write(&audio, b"not decoded during import").unwrap();
        let library = Library::open(&root.join("library.db")).unwrap();
        let category = library.add_category("Memes", None).unwrap();
        let id = library
            .add_sound(Sound {
                id: 0,
                name: "effect".into(),
                file_path: audio.to_string_lossy().into(),
                category_id: Some(category),
                profile_id: None,
                volume: 0.8,
                playback_mode: PlaybackMode::Overlay,
                loop_enabled: false,
                favorite: false,
                tags: String::new(),
                note: String::new(),
                sort_order: 0,
                play_count: 0,
                last_played_at: None,
            })
            .unwrap();
        library.remove_category(category).unwrap();
        assert_eq!(
            library
                .sounds(None, "")
                .unwrap()
                .into_iter()
                .find(|sound| sound.id == id)
                .unwrap()
                .category_id,
            None
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn recursive_import_filters_audio_extensions() {
        let root = temp_path("import");
        let nested = root.join("nested");
        fs::create_dir_all(&nested).unwrap();
        fs::write(root.join("one.mp3"), b"x").unwrap();
        fs::write(nested.join("two.ogg"), b"x").unwrap();
        fs::write(nested.join("ignored.txt"), b"x").unwrap();
        let library = Library::open(&root.join("library.db")).unwrap();
        assert_eq!(library.import_path(&root, None).unwrap().len(), 2);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn managed_import_copies_and_avoids_filename_collisions() {
        let root = temp_path("copy-import");
        let managed = root.join("managed");
        fs::create_dir_all(&root).unwrap();
        let source = root.join("same-name.mp3");
        fs::write(&source, b"source").unwrap();
        let library = Library::open(&root.join("library.db")).unwrap();
        library
            .import_path_with_mode(&source, None, Some(&managed))
            .unwrap();
        library
            .import_path_with_mode(&source, None, Some(&managed))
            .unwrap();
        assert!(managed.join("same-name.mp3").is_file());
        assert!(managed.join("same-name-1.mp3").is_file());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn manifest_round_trip_remaps_categories() {
        let source = temp_path("export-source");
        let target = temp_path("export-target");
        fs::create_dir_all(&source).unwrap();
        fs::create_dir_all(&target).unwrap();
        let source_library = Library::open(&source.join("library.db")).unwrap();
        let category = source_library.add_category("One shots", None).unwrap();
        let media = source.join("voice.mp3");
        fs::write(&media, b"x").unwrap();
        source_library
            .add_sound(Sound {
                id: 0,
                name: "voice".into(),
                file_path: media.to_string_lossy().into(),
                category_id: Some(category),
                profile_id: None,
                volume: 0.5,
                playback_mode: PlaybackMode::Queue,
                loop_enabled: false,
                favorite: true,
                tags: "test".into(),
                note: String::new(),
                sort_order: 1,
                play_count: 0,
                last_played_at: None,
            })
            .unwrap();
        source_library.set_hotkey(1, "ctrl+1").unwrap();
        let manifest = source_library.export_profile_json(None).unwrap();
        let target_library = Library::open(&target.join("library.db")).unwrap();
        assert_eq!(
            target_library.import_profile_json(&manifest, None).unwrap(),
            1
        );
        let restored = target_library.sounds(None, "voice").unwrap();
        assert_eq!(restored.len(), 1);
        assert_eq!(restored[0].playback_mode, PlaybackMode::Queue);
        assert!(restored[0].category_id.is_some());
        assert_eq!(
            target_library.hotkey(restored[0].id).unwrap().as_deref(),
            Some("ctrl+1")
        );
        let _ = fs::remove_dir_all(source);
        let _ = fs::remove_dir_all(target);
    }

    #[test]
    fn profiles_isolate_categories_and_sounds() {
        let root = temp_path("profiles");
        fs::create_dir_all(&root).unwrap();
        let library = Library::open(&root.join("library.db")).unwrap();
        let alpha = library.active_profile().unwrap().id;
        let beta = library.add_profile("beta", "").unwrap();
        let alpha_category = library.add_category("alpha", Some(alpha)).unwrap();
        let beta_category = library.add_category("beta", Some(beta)).unwrap();
        let alpha_media = root.join("alpha.wav");
        let beta_media = root.join("beta.wav");
        fs::write(&alpha_media, b"x").unwrap();
        fs::write(&beta_media, b"x").unwrap();
        for (name, path, category, profile) in [
            ("alpha", alpha_media, alpha_category, alpha),
            ("beta", beta_media, beta_category, beta),
        ] {
            library
                .add_sound(Sound {
                    id: 0,
                    name: name.into(),
                    file_path: path.to_string_lossy().into(),
                    category_id: Some(category),
                    profile_id: Some(profile),
                    volume: 0.8,
                    playback_mode: PlaybackMode::Overlay,
                    loop_enabled: false,
                    favorite: false,
                    tags: String::new(),
                    note: String::new(),
                    sort_order: 0,
                    play_count: 0,
                    last_played_at: None,
                })
                .unwrap();
        }
        assert_eq!(library.categories_in_profile(alpha).unwrap().len(), 1);
        assert_eq!(
            library.sounds_in_profile(beta, None, "").unwrap()[0].name,
            "beta"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn records_schema_migration_once() {
        let root = temp_path("migration");
        fs::create_dir_all(&root).unwrap();
        let first = Library::open(&root.join("library.db")).unwrap();
        assert_eq!(first.schema_version().unwrap(), 2);
        drop(first);
        assert_eq!(
            Library::open(&root.join("library.db"))
                .unwrap()
                .schema_version()
                .unwrap(),
            2
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn app_settings_persist_after_v2_migration() {
        let root = temp_path("settings");
        fs::create_dir_all(&root).unwrap();
        let library = Library::open(&root.join("library.db")).unwrap();
        library.set_setting("last_device", "BlackHole 2ch").unwrap();
        assert_eq!(
            library.setting("last_device").unwrap().as_deref(),
            Some("BlackHole 2ch")
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn favorites_and_recents_are_profile_scoped() {
        let root = temp_path("filters");
        fs::create_dir_all(&root).unwrap();
        let library = Library::open(&root.join("library.db")).unwrap();
        let profile = library.active_profile().unwrap().id;
        let media = root.join("favorite.ogg");
        fs::write(&media, b"x").unwrap();
        let id = library
            .add_sound(Sound {
                id: 0,
                name: "favorite".into(),
                file_path: media.to_string_lossy().into(),
                category_id: None,
                profile_id: Some(profile),
                volume: 0.8,
                playback_mode: PlaybackMode::Overlay,
                loop_enabled: false,
                favorite: true,
                tags: String::new(),
                note: String::new(),
                sort_order: 0,
                play_count: 0,
                last_played_at: None,
            })
            .unwrap();
        assert_eq!(
            library
                .favorite_sounds_in_profile(profile, "")
                .unwrap()
                .len(),
            1
        );
        assert!(library
            .recent_sounds_in_profile(profile, "")
            .unwrap()
            .is_empty());
        library.record_play(id, "test").unwrap();
        assert_eq!(
            library.recent_sounds_in_profile(profile, "").unwrap()[0].id,
            id
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_bare_tui_key_bindings() {
        let root = temp_path("reserved-hotkey");
        fs::create_dir_all(&root).unwrap();
        let library = Library::open(&root.join("library.db")).unwrap();
        let media = root.join("sound.wav");
        fs::write(&media, b"x").unwrap();
        let id = library
            .add_sound(Sound {
                id: 0,
                name: "sound".into(),
                file_path: media.to_string_lossy().into(),
                category_id: None,
                profile_id: None,
                volume: 0.8,
                playback_mode: PlaybackMode::Overlay,
                loop_enabled: false,
                favorite: false,
                tags: String::new(),
                note: String::new(),
                sort_order: 0,
                play_count: 0,
                last_played_at: None,
            })
            .unwrap();
        assert!(library.set_hotkey(id, "b").is_err());
        assert!(library.set_hotkey(id, "ctrl+b").is_ok());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn cleans_legacy_reserved_bindings() {
        let root = temp_path("legacy-hotkey");
        fs::create_dir_all(&root).unwrap();
        let library = Library::open(&root.join("library.db")).unwrap();
        let now = Utc::now().to_rfc3339();
        library.conn.execute("INSERT INTO hotkeys(sound_id,action,hotkey,created_at,updated_at) VALUES(1,'play','b',?1,?1)", [now]).unwrap();
        assert_eq!(library.clear_reserved_tui_hotkeys().unwrap(), 1);
        assert!(library.hotkey(1).unwrap().is_none());
        let _ = fs::remove_dir_all(root);
    }
}
