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
}
