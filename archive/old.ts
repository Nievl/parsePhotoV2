class old {
  constructor() {
    console.log('old');
  }

  /**
   * Извлекает URL изображений и видео из HTML-страницы.
   *
   * @param {HTMLElement} page - HTML-страница, из которой извлекаются медиа-файлы.
   * @param {boolean} absoluteOnly - Фильтровать только абсолютные URL (начинающиеся с `http://` или `https://`).
   * @returns {Set<string>} Уникальный список URL медиа-файлов.
   */
  private getMediaUrls(page: string, absoluteOnly: boolean = false, domain?: string): Set<string> {
    const root = parse(page);
    const urls = new Set<string>();
    const media = root.querySelectorAll('img, video');

    for (const element of media) {
      const src = element.getAttribute('src');
      if (src) {
        if (!absoluteOnly || src.startsWith('http://') || src.startsWith('https://')) {
          if (domain && !src.startsWith('http')) {
            urls.add(domain + src);
          } else {
            urls.add(src);
          }
        }
      }
    }

    return urls;
  }

  /** Генерирует URL высокого разрешения */
  private async getHighResUrl(url: string): Promise<string> {
    const highResUrl = url.replace('/a/604/', '/a/1280/');
    if (await this.isImageAccessible(highResUrl)) {
      return highResUrl;
    } else {
      return url;
    }
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
