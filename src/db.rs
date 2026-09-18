use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct Memory {
    pub id: String,
    pub project: String,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
    pub created_at: String,
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }

        let conn = Connection::open(path)
            .with_context(|| format!("failed to open SQLite database at {}", path.display()))?;

        conn.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA foreign_keys = ON;
            PRAGMA busy_timeout = 5000;

            CREATE TABLE IF NOT EXISTS memories (
                row_id      INTEGER PRIMARY KEY AUTOINCREMENT,
                id          TEXT NOT NULL UNIQUE
                            DEFAULT (lower(hex(randomblob(16)))),
                project     TEXT NOT NULL,
                title       TEXT NOT NULL,
                body        TEXT NOT NULL,
                tags        TEXT NOT NULL DEFAULT '',
                created_at  TEXT NOT NULL
                            DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
            ) STRICT;

            CREATE INDEX IF NOT EXISTS idx_memories_project_created
                ON memories(project, created_at DESC);

            CREATE VIRTUAL TABLE IF NOT EXISTS memories_fts USING fts5(
                title,
                body,
                tags,
                content='memories',
                content_rowid='row_id',
                tokenize='unicode61 remove_diacritics 2'
            );

            CREATE TRIGGER IF NOT EXISTS memories_ai
            AFTER INSERT ON memories BEGIN
                INSERT INTO memories_fts(rowid, title, body, tags)
                VALUES (new.row_id, new.title, new.body, new.tags);
            END;

            CREATE TRIGGER IF NOT EXISTS memories_ad
            AFTER DELETE ON memories BEGIN
                INSERT INTO memories_fts(memories_fts, rowid, title, body, tags)
                VALUES ('delete', old.row_id, old.title, old.body, old.tags);
            END;

            CREATE TRIGGER IF NOT EXISTS memories_au
            AFTER UPDATE OF title, body, tags ON memories BEGIN
                INSERT INTO memories_fts(memories_fts, rowid, title, body, tags)
                VALUES ('delete', old.row_id, old.title, old.body, old.tags);
                INSERT INTO memories_fts(rowid, title, body, tags)
                VALUES (new.row_id, new.title, new.body, new.tags);
            END;
            "#,
        )
        .context("failed to initialize database schema")?;

        Ok(Self { conn })
    }

    pub fn remember(
        &self,
        project: &str,
        title: &str,
        body: &str,
        tags: &[String],
    ) -> Result<Memory> {
        let tags_text = tags.join(",");

        self.conn.execute(
            "INSERT INTO memories(project, title, body, tags) VALUES (?1, ?2, ?3, ?4)",
            params![project, title, body, tags_text],
        )?;

        let row_id = self.conn.last_insert_rowid();
        self.get_by_row_id(row_id)
    }

    pub fn search(&self, project: &str, query: &str, limit: u32) -> Result<Vec<Memory>> {
        let fts_query = make_fts_query(query);
        let mut stmt = self.conn.prepare(
            r#"
            SELECT m.id, m.project, m.title, m.body, m.tags, m.created_at
            FROM memories_fts
            JOIN memories AS m ON m.row_id = memories_fts.rowid
            WHERE memories_fts MATCH ?1
              AND m.project = ?2
            ORDER BY bm25(memories_fts, 5.0, 1.0, 0.5), m.created_at DESC
            LIMIT ?3
            "#,
        )?;

        let rows = stmt.query_map(params![fts_query, project, i64::from(limit)], map_memory)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .context("failed to collect search results")
    }

    fn get_by_row_id(&self, row_id: i64) -> Result<Memory> {
        self.conn
            .query_row(
                "SELECT id, project, title, body, tags, created_at FROM memories WHERE row_id = ?1",
                [row_id],
                map_memory,
            )
            .context("failed to read inserted memory")
    }
}

fn map_memory(row: &rusqlite::Row<'_>) -> rusqlite::Result<Memory> {
    let tags: String = row.get(4)?;
    Ok(Memory {
        id: row.get(0)?,
        project: row.get(1)?,
        title: row.get(2)?,
        body: row.get(3)?,
        tags: tags
            .split(',')
            .filter(|tag| !tag.is_empty())
            .map(str::to_owned)
            .collect(),
        created_at: row.get(5)?,
    })
}

fn make_fts_query(input: &str) -> String {
    input
        .split_whitespace()
        .filter(|token| !token.is_empty())
        .map(|token| format!("\"{}\"", token.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(" OR ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remembers_and_searches() {
        let db = Database::open(Path::new(":memory:")).unwrap();
        db.remember(
            "demo",
            "SQLite decision",
            "Use WAL mode for concurrent readers",
            &["architecture".into(), "sqlite".into()],
        )
        .unwrap();

        let results = db.search("demo", "concurrent SQLite", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "SQLite decision");
    }

    #[test]
    fn isolates_projects() {
        let db = Database::open(Path::new(":memory:")).unwrap();
        db.remember("one", "Decision", "Use SQLite", &[]).unwrap();
        assert!(db.search("two", "SQLite", 10).unwrap().is_empty());
    }
}
