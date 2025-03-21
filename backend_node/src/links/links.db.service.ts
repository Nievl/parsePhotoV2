import { HttpException, HttpStatus, Injectable } from '@nestjs/common';
import { AsyncDatabase } from 'promised-sqlite3';
import { iResult } from '../entities/common';
import { CreateLinkDto, Link, LinkDto, UpdateLinkDto } from '../entities/link.entity';
import { dateConvert, resultMaker } from '../helpers/common';

@Injectable()
export default class LinksDbService {
  public async getAll(isReachable: boolean, showDuplicate: boolean): Promise<Link[]> {
    const db = await AsyncDatabase.open(process.env.DB_NAME);
    let query: string;

    if (showDuplicate) {
      query = `
          SELECT l.*, d.path AS duplicate_path 
          FROM links AS l
          LEFT JOIN links AS d
          ON l.duplicate_id = d.id
          WHERE l.is_reachable = ? AND l.duplicate_id IS NOT NULL 
          ORDER BY l.is_downloaded
        `;
    } else {
      query = `
        SELECT * 
        FROM links 
        WHERE is_reachable = ? AND duplicate_id IS NULL 
        ORDER BY is_downloaded
        `;
    }

    const result = await db.all<Link>(query, [isReachable]);

    await db.close();

    const mappedResult = result.map((row) => new LinkDto(row));
    return mappedResult;
  }

  public async createOne({ path, name }: CreateLinkDto): Promise<iResult> {
    const db = await AsyncDatabase.open(process.env.DB_NAME);
    try {
      const result = await db.run(
        `
        INSERT INTO links (path, name, is_reachable)
        VALUES (?, ?, 1)`,
        [path.trim(), name],
      );

      await db.close();
      if (result.changes === 1) {
        return resultMaker('One path created');
      }
      return resultMaker('No path created');
    } catch (error) {
      if (error.code === 'SQLITE_CONSTRAINT') {
        throw new HttpException('Already exist ', HttpStatus.BAD_REQUEST);
      }
      throw new HttpException('can not make transaction', HttpStatus.INTERNAL_SERVER_ERROR);
    }
  }

  public async remove(id: string): Promise<iResult> {
    const db = await AsyncDatabase.open(process.env.DB_NAME);
    try {
      const result = await db.run(
        `
        DELETE FROM links
        WHERE id=?`,
        [id],
      );
      await db.close();
      if (result.changes === 1) {
        return resultMaker('One path removed');
      }
      return resultMaker('No one path removed');
    } catch (error) {
      throw new HttpException('can not make transaction', HttpStatus.INTERNAL_SERVER_ERROR);
    }
  }

  public async getOne(id: string): Promise<Link> {
    const db = await AsyncDatabase.open(process.env.DB_NAME);
    try {
      const result = await db.get<Link>(
        `
        SELECT *
        FROM links
        WHERE id=?`,
        [id],
      );
      await db.close();

      return new LinkDto(result);
    } catch (error) {
      throw new HttpException('can not make transaction', HttpStatus.INTERNAL_SERVER_ERROR);
    }
  }

  public async tagUnreachable(id: string, isReachable: boolean): Promise<iResult> {
    const db = await AsyncDatabase.open(process.env.DB_NAME);
    try {
      const result = await db.run(
        `
        UPDATE links
        SET is_reachable = ?,  date_update = ? 
        WHERE id=?`,
        [isReachable, dateConvert(), id],
      );
      await db.close();

      if (result.changes === 1) {
        return resultMaker(`One path tag as ${isReachable ? '' : 'un'}reachable`);
      }
      return resultMaker('No path tagged');
    } catch (error) {
      throw new HttpException('can not make transaction', HttpStatus.INTERNAL_SERVER_ERROR);
    }
  }

  public async addDuplicate(linkId: string, duplicateId: string): Promise<iResult> {
    const db = await AsyncDatabase.open(process.env.DB_NAME);
    try {
      const result = await db.run(
        `
        UPDATE links
        SET duplicate_id = ?, date_update = ?
        WHERE id = ?`,
        [duplicateId, dateConvert(), linkId],
      );
      await db.close();

      if (result.changes === 1) {
        return resultMaker(`Link ${linkId} tagged as duplicate of ${duplicateId}`);
      } else {
        return resultMaker('No path tagged as duplicate');
      }
    } catch (error) {
      throw new HttpException('can not make transaction', HttpStatus.INTERNAL_SERVER_ERROR);
    }
  }

  public async updateFilesNumber(
    id: number,
    { mediafiles, downloadedMediafiles, isDownloaded, progress }: UpdateLinkDto,
  ): Promise<iResult> {
    const db = await AsyncDatabase.open(process.env.DB_NAME);
    try {
      const result = await db.run(
        `
          UPDATE links 
          SET mediafiles = ?, downloaded_mediafiles = ?, is_downloaded = ?, progress = ?, date_update = ? 
          WHERE id = ?`,
        [mediafiles, downloadedMediafiles, isDownloaded, progress, dateConvert(), id],
      );
      await db.close();

      if (result.changes === 1) {
        return resultMaker('One path updated');
      }
      return resultMaker('No path updated');
    } catch (error) {
      throw new HttpException('can not make transaction', HttpStatus.INTERNAL_SERVER_ERROR);
    }
  }
}
