import { Injectable } from '@nestjs/common';
import { AsyncDatabase } from 'promised-sqlite3';
import { iResult } from 'src/entities/common';
import { CreateMediafileDto, Mediafile, MediafileDto } from 'src/entities/mediafiles.entity';
import { resultMaker } from 'src/helpers/common';
import { open } from 'sqlite';
import { Database } from 'sqlite3';

@Injectable()
export class MediafilesDbService {
  public async createOne(dto: CreateMediafileDto): Promise<iResult> {
    const db = await open({
      filename: process.env.DB_NAME,
      driver: Database,
    });

    const { name, path, hash, size, linkId } = dto;

    await db.getDatabaseInstance().serialize(() => {
      db.run(
        `
        INSERT OR IGNORE INTO mediafiles (path, name, hash, size)
        VALUES (?, ?, ?, ?);
      `,
        [path, name, hash, size],
      );
      db.run(
        `
        INSERT OR IGNORE INTO mediafiles_links (link_id, mediafile_id)
          VALUES (?, 
            (SELECT id as mediafile_id
            FROM mediafiles
            WHERE path = ?)
          );
      `,
        [linkId, path],
      );
    });

    await db.close();

    return resultMaker('ok');
  }

  public async remove(id: string): Promise<iResult> {
    const db = await AsyncDatabase.open(process.env.DB_NAME);
    const result = await db.run(
      `
      DELETE FROM mediafiles WHERE id = ?;
      DELETE FROM mediafiles_links WHERE mediafile_id = ?;
      `,
      [id, id],
    );
    await db.close();
    if (result.changes == 1) {
      return resultMaker('One mediafile removed');
    } else {
      return resultMaker('No mediafile removed"');
    }
  }

  public async getAllByLinkId(linkId: string): Promise<Mediafile[]> {
    const db = await AsyncDatabase.open(process.env.DB_NAME);
    const result = await db.all(
      `
        SELECT m.id, m.path, m.name, m.hash, m.size, m.date_added
        FROM mediafiles m
        JOIN mediafiles_links ml ON m.id = ml.mediafile_id
        WHERE ml.link_id = ?;
      `,
      [linkId],
    );
    await db.close();

    const mappedResult = result.map((row) => new MediafileDto(row));
    return mappedResult;
  }
}
