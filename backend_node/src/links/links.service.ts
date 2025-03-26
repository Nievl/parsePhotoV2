import { HttpException, HttpStatus, Injectable, Logger } from '@nestjs/common';
import axios from 'axios';
import * as fs from 'fs';
import * as path from 'path';
import { iResult } from '../entities/common';
import { CreateLinkDto, Link } from '../entities/link.entity';
import { resultMaker } from '../helpers/common';

import LinksDbService from './links.db.service';
import { MediafilesService } from '../mediafiles/mediafiles.service';
import { CreateMediafileDto } from '../entities/mediafiles.entity';
import { getMediaUrls, getHighResUrl } from '../napi/';

const EXTENSIONS = ['jpeg', 'jpg', 'mp4', 'png', 'gif', 'webp'];
const checkUrl = (url: string): string[] | null => url.trim().match(/(http[s]?:\/\/[^\/\s]+\/)(.*)/);
const MAX_CONCURRENT_DOWNLOADS = 5; // Ограничение параллельных загрузок

@Injectable()
export class LinksService {
  constructor(
    private linksDbService: LinksDbService,
    private mediafilesService: MediafilesService,
  ) {}

  /**
   * Создание нового объекта ссылки.
   * @param filePath - путь, переданный для создания ссылки.
   * @returns Результат создания ссылки.
   */
  createOne({ path: filePath }: CreateLinkDto): Promise<iResult> {
    const url = checkUrl(filePath);
    if (url && url.length === 3) {
      const pathAfterDomain = url[2];
      const finalName = pathAfterDomain.replace(/^[^/]+\//, '');

      return this.linksDbService.createOne(filePath, finalName);
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
    const isTelegraph = link.path.includes('telegra.ph');
    const urls = getMediaUrls(page, isOsosedkiDomain, isTelegraph ? 'https://telegra.ph' : undefined);
    const dirPath = await this.createDirectory(link.name);
    const downloadQueue: (() => Promise<CreateMediafileDto | null>)[] = [];
    let downloadedCount = 0;
    let totalFiles = 0;

    for (const url of urls) {
      const cleanUrl = url.split('?')[0];
      const ext = path.extname(cleanUrl).replace('.', '');

      if (!EXTENSIONS.includes(ext)) {
        Logger.warn(`${link.name} is not picture`);
        continue;
      }
      totalFiles++;

      const fileName = cleanUrl.replace(/.+\//g, '');
      const pathName = path.join(dirPath, fileName);
      const useRootUrl = !url.match(/https?:?\/\//);
      const fullUrl = useRootUrl ? link.path + url : url;

      if (fs.existsSync(pathName)) {
        downloadedCount++;
        continue;
      }

      // Добавляем задачу скачивания в очередь
      downloadQueue.push(async () => {
        const newUrl = await getHighResUrl(fullUrl);
        return this.mediafilesService.downloadFile(newUrl, pathName, id);
      });
    }

    const downloadedMediafiles = (await this.processQueue(downloadQueue)).filter((m) => m !== null);

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

  /**
   * Проверяет статус загрузки файлов для указанной ссылки, включая проверку наличия файлов в директории и доступности страницы.
   *
   * @param {string} id - ID ссылки, для которой нужно проверить статус загрузки.
   * @returns {Promise<iResult>} Промис, который возвращает результат операции, включая статус загрузки и сообщение.
   *
   * @throws {Error} Если возникает ошибка при попытке получить страницу или при операциях с файловой системой.
   *
   * @description
   * Этот метод выполняет следующие шаги:
   * 1. Извлекает ссылку из базы данных по предоставленному ID.
   * 2. Проверяет, существует ли директория, связанная с этой ссылкой.
   * 3. Попытка получить HTML-страницу, ассоциированную с ссылкой.
   * 4. Проверяет, является ли домен ссылкой на сайт "Ососедки".
   * 5. В зависимости от наличия директории и страницы, выполняет следующие действия:
   *    - Если директория не существует и страница не найдена, возвращает сообщение об ошибке.
   *    - Если директория существует, но страница не найдена, проверяет количество файлов в директории и обновляет статус в базе данных.
   *    - Если директория и страница существуют, сравнивает количество файлов в директории и медиаресурсов на странице, обновляет прогресс и статус загрузки.
   *    - Если директория не существует, но страница найдена, обновляет статус в базе данных о том, что файлы не были загружены.
   *
   * Пример использования:
   * ```typescript
   * const result = await checkDownloaded('linkId123');
   * console.log(result.message); // Выводит результат с информацией о статусе загрузки.
   * ```
   */
  public async checkDownloaded(id: string): Promise<iResult> {
    const link = await this.linksDbService.getOne(id);

    const dirPath = path.join(__dirname, '../../../result', link.name);
    let page: string | null = null;
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
      const mediafiles = getMediaUrls(page, isOsosedkiDomain).length;
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
      const mediafiles = getMediaUrls(page, isOsosedkiDomain).length;

      await this.linksDbService.updateFilesNumber(link.id, {
        downloadedMediafiles: 0,
        isDownloaded: false,
        mediafiles,
        progress: 0,
      });
      return resultMaker(`id: ${id}, Not downloaded yet`);
    }
  }

  /**
   * Сканирует директорию на наличие медиафайлов, связанных с определенной ссылкой, и обрабатывает новые файлы.
   *
   * @param {string} id - ID ссылки, для которой нужно просканировать файлы.
   * @returns {Promise<iResult>} Промис, который возвращает результат операции, включая статус и соответствующие сообщения.
   *
   * @throws {Error} Если возникает ошибка при чтении файлов, вычислении хешей или при операциях с базой данных.
   *
   * @description
   * Этот метод выполняет следующие шаги:
   * 1. Извлекает ссылку из базы данных по предоставленному ID.
   * 2. Проверяет, существует ли директория, связанная с ссылкой.
   * 3. Если директория не существует, возвращает результат, указывающий на это.
   * 4. Если директория существует, читает все файлы в ней.
   * 5. Сравнивает файлы в директории с уже обработанными (на основе существующих медиафайлов в базе данных).
   * 6. Для новых файлов вычисляет хеш и размер, а затем создает новую запись в базе данных.
   * 7. Возвращает результат, указывающий количество файлов в директории.
   *
   */
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
   * Загружает и парсит HTML-контент по указанному URL.
   *
   * @param {string} url - URL веб-страницы, которую нужно загрузить.
   * @returns {Promise<HTMLElement>} Промис, который возвращает распарсенный HTML-контент.
   * @throws {Error} В случае ошибки запроса или проблемы с парсингом.
   */
  private async getPage(url: string): Promise<string> {
    const response = await axios.get<string>(url);

    return response.data;
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

  /**
   * Обрабатывает очередь асинхронных задач с ограничением числа одновременных потоков.
   *
   * @param queue - Массив функций, каждая из которых возвращает `Promise<CreateMediafileDto | null>`.
   * @returns Массив результатов выполнения всех задач.
   *
   * ⚡ Алгоритм работы:
   * 1. Поддерживает ограниченное количество активных задач (`MAX_CONCURRENT_DOWNLOADS`).
   * 2. Запускает новые задачи, пока есть свободные слоты.
   * 3. Ждет завершения одной из активных задач (использует `Promise.race`).
   * 4. После завершения задачи освобождает слот и добавляет новую из очереди.
   * 5. Повторяет процесс, пока не выполнит все задачи.
   */
  private async processQueue(
    queue: (() => Promise<CreateMediafileDto | null>)[],
  ): Promise<(CreateMediafileDto | null)[]> {
    const activeTasks: Promise<CreateMediafileDto | null>[] = [];
    const results: (CreateMediafileDto | null)[] = [];

    while (queue.length > 0 || activeTasks.length > 0) {
      while (activeTasks.length < MAX_CONCURRENT_DOWNLOADS && queue.length > 0) {
        const task = queue.shift();
        if (task) {
          const taskPromise = task().then((res) => {
            activeTasks.splice(activeTasks.indexOf(taskPromise), 1);
            results.push(res);
            return res;
          });
          activeTasks.push(taskPromise);
        }
      }
      await Promise.race(activeTasks);
    }

    return results;
  }
}
