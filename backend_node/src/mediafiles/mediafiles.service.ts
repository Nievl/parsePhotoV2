import { iResult } from 'src/entities/common';
import { CreateMediafileDto, Mediafile } from 'src/entities/mediafiles.entity';
import { MediafilesDbService } from './mediafiles.db.service';
import { Injectable, Logger } from '@nestjs/common';
import * as crypto from 'crypto';
import * as fs from 'fs';
import * as path from 'path';
import axios from 'axios';

@Injectable()
export class MediafilesService {
  constructor(private readonly mediafilesDbService: MediafilesDbService) {}

  public async createOne(dto: CreateMediafileDto): Promise<iResult> {
    return this.mediafilesDbService.createOne(dto);
  }

  public async remove(id: string): Promise<iResult> {
    return this.mediafilesDbService.remove(id);
  }

  public async getAllByLinkId(linkId: string): Promise<Mediafile[]> {
    return this.mediafilesDbService.getAllByLinkId(linkId);
  }

  public async downloadFile(url: string, filePath: string, linkId: string): Promise<CreateMediafileDto | null> {
    try {
      const writer = fs.createWriteStream(filePath);
      const response = await axios.get(url, {
        responseType: 'stream',
        headers: {
          'User-Agent':
            'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36',
        },
      });

      let totalSize = 0;

      response.data.on('data', (chunk: Buffer) => (totalSize += chunk.length));

      response.data.pipe(writer);

      const size = await new Promise<number>((resolve, reject) => {
        writer.on('finish', () => resolve(totalSize));
        writer.on('error', reject);
      });

      const hash = await this.getHashByPath(filePath);
      const name = path.basename(filePath);

      return {
        name,
        path: filePath,
        hash,
        size,
        linkId,
      };
    } catch (error) {
      let message;
      if (error instanceof Error) {
        message = error.message;
      } else {
        message = String(error);
      }
      Logger.error(`\nurl: ${url}\n`, message);
      return null;
    }
  }

  public async getHashByPath(path: string): Promise<string> {
    const fileBuffer = await fs.promises.readFile(path);
    const hashSum = crypto.createHash('sha256');
    hashSum.update(fileBuffer);
    return hashSum.digest('hex');
  }
}
