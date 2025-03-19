import { Logger } from '@nestjs/common';
import * as fs from 'fs';
import * as path from 'path';
import sqlite3 from 'sqlite3';

export function checkAndCreateTables(): void {
  Logger.log('read db file, process.env.DB_NAME: ', process.env.DB_NAME);

  const dbPath = path.join(__dirname, `../${process.env.DB_NAME}`);

  if (!fs.existsSync(dbPath)) {
    throw new Error(`${dbPath} does not exist`);
  }

  const db = new sqlite3.Database(process.env.DB_NAME, (err) => {
    if (err) {
      Logger.error('Could not connect to database', err);
    } else {
      Logger.log('Connected to database');
    }
  });

  db.serialize(function () {
    // Create a table
    db.run(`
        CREATE TABLE IF NOT EXISTS links (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT NOT NULL UNIQUE,
            name TEXT,
            is_downloaded BOOLEAN NOT NULL DEFAULT 1,
            progress INTEGER DEFAULT 0,
            downloaded_mediafiles INTEGER DEFAULT 0,
            mediafiles INTEGER DEFAULT 0,
            date_update DATETIME DEFAULT CURRENT_TIMESTAMP,
            date_create DATETIME DEFAULT CURRENT_TIMESTAMP,
            is_reachable BOOLEAN NOT NULL DEFAULT 0,
            duplicate_id INTEGER DEFAULT NULL
        )
    `);

    db.run(`
        CREATE TABLE IF NOT EXISTS mediafiles (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                hash TEXT NOT NULL,
                size INTEGER NOT NULL,
                date_added DATETIME DEFAULT CURRENT_TIMESTAMP
            )
    `);

    db.run(`CREATE TABLE IF NOT EXISTS mediafiles_links (
                link_id INTEGER NOT NULL,
                mediafile_id INTEGER NOT NULL,
                FOREIGN KEY (link_id) REFERENCES links(id) ON DELETE CASCADE,
                FOREIGN KEY (mediafile_id) REFERENCES mediafiles(id) ON DELETE CASCADE,
                PRIMARY KEY (link_id, mediafile_id)
            )
    `);
  });

  db.close();
}
