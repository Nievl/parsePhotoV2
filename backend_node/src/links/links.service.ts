import { HttpException, HttpStatus, Injectable, Logger } from '@nestjs/common';
import axios from 'axios';
import * as fs from 'fs';
import parse, { HTMLElement } from 'node-html-parser';
import * as path from 'path';
import { iResult } from 'src/entities/common';
import { CreateLinkDto, Link } from 'src/entities/link.entity';
import { checkUrl, resultMaker } from 'src/helpers/common';

import LinksDbService from './links.db.service';
import { MediafilesService } from 'src/mediafiles/mediafiles.service';
import { CreateMediafileDto } from 'src/entities/mediafiles.entity';

const EXTENSIONS = ['jpeg', 'jpg', 'mp4', 'png', 'gif', 'webp'];

@Injectable()
export class LinksService {
  constructor(private linksDbService: LinksDbService, private mediafilesService: MediafilesService) {}
  createOne({ path: filePath }: CreateLinkDto): Promise<iResult> {
    const url = checkUrl(filePath);
    if (url && url.length === 3) {
      return this.linksDbService.createOne({ path: filePath, name: url[2] });
    } else {
      throw new HttpException('path is not url', HttpStatus.BAD_REQUEST);
    }
  }

  getAll(isReachable: boolean, showDuplicate: boolean): Promise<Link[]> {
    return this.linksDbService.getAll(isReachable, showDuplicate);
  }

  public remove(id: string): Promise<iResult> {
    return this.linksDbService.remove(id);
  }

  public tagUnreachable(id: string, isReachable: boolean): Promise<iResult> {
    return this.linksDbService.tagUnreachable(id, isReachable);
  }

  /**
   * Загружает изображения по указанному `id`, проверяет доступность
   * более качественной версии и сохраняет файлы на диск.
   *
   * @param {string} id - Уникальный идентификатор ссылки на страницу.
   * @returns {Promise<iResult>} Объект с результатом загрузки.
   */
  public async downloadFiles(id: string): Promise<iResult> {
    const link = await this.linksDbService.getOne(id);
    const page = await this.getPage(link.path);
    const isOsosedkiDomain = this.isOsosedkiDomain(link.path);
    const urls = this.getMediaUrls(page, isOsosedkiDomain);
    const dirPath = await this.createDirectory(link.name);
    const requests: (CreateMediafileDto | null)[] = [];
    let downloadedCount = 0;
    let totalFiles = 0;

    for (const url of Array.from(urls)) {
      const ext = (/[^.]+$/.exec(url) ?? [])[0] ?? '';
      const fileName = url.replace(/.+\//g, '');
      const pathName = path.join(dirPath, fileName);
      const useRootUrl = !url.match(/https?:?\/\//);
      const fullUrl = useRootUrl ? link.path + url : url;
      if (EXTENSIONS.includes(ext)) {
        totalFiles++;

        if (!fs.existsSync(pathName)) {
          if (isOsosedkiDomain) {
            const highResUrl = this.getHighResUrl(fullUrl);
            if (await this.isImageAccessible(highResUrl)) {
              requests.push(await this.mediafilesService.downloadFile(highResUrl, pathName, id));
            } else {
              requests.push(await this.mediafilesService.downloadFile(fullUrl, pathName, id));
            }
          } else {
            requests.push(await this.mediafilesService.downloadFile(fullUrl, pathName, id));
          }
        } else {
          downloadedCount++;
        }
      } else {
        Logger.warn(`${link.name} is not picture`);
      }
    }

    const downloadedMediafiles = requests.filter((m) => m !== null);

    await Promise.all(downloadedMediafiles.map((m) => this.mediafilesService.createOne(m)));

    downloadedCount += downloadedMediafiles.length;
    const progress = Math.round((downloadedCount * 100) / totalFiles);
    const isDownloaded = downloadedCount === totalFiles;

    await this.linksDbService.updateFilesNumber(link.id, {
      downloadedMediafiles: downloadedCount,
      isDownloaded,
      mediafiles: totalFiles,
      progress,
    });

    return { success: true, message: `downloaded ${downloadedMediafiles}` };
  }

  public async checkDownloaded(id: string): Promise<iResult> {
    const link = await this.linksDbService.getOne(id);

    const dirPath = path.join(__dirname, '../../../result', link.name);
    let page: HTMLElement | null = null;
    const dir = fs.existsSync(dirPath);
    try {
      page = await this.getPage(link.path);
    } catch (error) {}

    const isOsosedkiDomain = this.isOsosedkiDomain(link.path);

    if (!dir && !page) {
      return resultMaker(`${dirPath}  is not exists and \n page not found`);
    } else if (dir && !page) {
      const files = await fs.promises.readdir(dirPath);
      const mediafiles = files.length;
      if (files.length > 0) {
        await this.linksDbService.updateFilesNumber(link.id, {
          downloadedMediafiles: mediafiles,
          isDownloaded: true,
          mediafiles,
          progress: 100,
        });
      } else {
        return resultMaker(`${mediafiles} files in ${dirPath}  directory,\n page not found`);
      }
    } else if (dir && page) {
      const files = fs.readdirSync(dirPath);
      const mediafiles = this.getMediaUrls(page, isOsosedkiDomain).size;
      const progress = Math.round((files.length * 100) / mediafiles);
      const isDownloaded = files.length === mediafiles;
      await this.linksDbService.updateFilesNumber(link.id, {
        downloadedMediafiles: files.length,
        isDownloaded,
        mediafiles,
        progress,
      });
      return resultMaker(`dowloaded ${files.length} from ${mediafiles}`);
    } else if (!dir && page) {
      const mediafiles = this.getMediaUrls(page, isOsosedkiDomain).size;

      await this.linksDbService.updateFilesNumber(link.id, {
        downloadedMediafiles: 0,
        isDownloaded: false,
        mediafiles,
        progress: 0,
      });
      return resultMaker(`id: ${id}, Not downloaded yet`);
    }
  }

  public async scanFilesForLink(id: string): Promise<iResult> {
    const link = await this.linksDbService.getOne(id);

    const dirPath = path.join(__dirname, '../../../result', link.name).normalize();

    const dir = fs.existsSync(dirPath);
    if (!dir) {
      return resultMaker(`${dirPath}  is not exists`);
    }

    const files = await fs.promises.readdir(dirPath);
    const existedMediafiles = await this.mediafilesService.getAllByLinkId(id);
    const existedMediafilesPathSet = new Set(existedMediafiles.map((i) => i.name));
    const requests: Promise<iResult>[] = files.map(async (fileName) => {
      const pathName = path.join(dirPath, fileName);
      if (!existedMediafilesPathSet.has(pathName)) {
        const hash = await this.mediafilesService.getHashByPath(pathName);
        const stats = await fs.promises.stat(pathName);

        return this.mediafilesService.createOne({
          name: fileName,
          hash,
          linkId: id,
          path: path.join('result', link.name, fileName),
          size: stats.size,
        });
      }
    });

    await Promise.all(requests);

    return resultMaker(`${files.length} files in ${dirPath}  directory`);
  }

  public addDuplicate(linkId: string, duplicateId: string): Promise<iResult> {
    return this.linksDbService.addDuplicate(linkId, duplicateId);
  }

  /**
   * Извлекает URL изображений и видео из HTML-страницы.
   *
   * @param {HTMLElement} page - HTML-страница, из которой извлекаются медиа-файлы.
   * @param {boolean} absoluteOnly - Фильтровать только абсолютные URL (начинающиеся с `http://` или `https://`).
   * @returns {Set<string>} Уникальный список URL медиа-файлов.
   */
  private getMediaUrls(page: HTMLElement, absoluteOnly: boolean = false): Set<string> {
    const media = page.querySelectorAll('img, video');
    const urls = Array.from(media)
      .map((i) => i.getAttribute('src'))
      .filter((i): i is string => i !== null && i !== undefined);

    return new Set(absoluteOnly ? urls.filter((url) => url.startsWith('http://') || url.startsWith('https://')) : urls);
  }

  private async getPage(url: string): Promise<HTMLElement> {
    const response = await axios.get<string>(url);

    return parse(response.data);
  }

  private async createDirectory(linkName: string): Promise<string> {
    const dirPath = path.join(__dirname, '../../../result', linkName);

    if (!fs.existsSync(dirPath)) {
      Logger.log(dirPath + ' is not exists');
      try {
        await fs.promises.mkdir(dirPath, { recursive: true });
      } catch (error) {
        Logger.error('error: ', error.message);
      }
    }
    return dirPath;
  }

  /** Проверяет, принадлежит ли URL домену ososedki.com */
  private isOsosedkiDomain(url: string): boolean {
    return url.includes('ososedki.com');
  }

  /** Генерирует URL высокого разрешения */
  private getHighResUrl(url: string): string {
    return url.replace('/a/604/', '/a/1280/');
  }

  /** Проверяет, доступно ли изображение высокого разрешения   */
  private async isImageAccessible(url: string): Promise<boolean> {
    try {
      const response = await axios.head(url, {
        timeout: 5000,
        headers: {
          'User-Agent':
            'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36',
        },
      });
      return response.status === 200;
    } catch (e) {
      Logger.log(`${url} has no high res`);
      return false;
    }
  }
}
