export interface Link extends CreateLinkDto, UpdateLinkDto {
  id: number;
  dateCreate: Date;
  dateUpdate: Date;
  isReachable: boolean;
  name: string;
}

export class CreateLinkDto {
  path: string;
}

export class UpdateLinkDto {
  isDownloaded: boolean;
  progress: number;
  mediafiles: number;
  downloadedMediafiles: number;
}

export class LinkDto implements Link {
  id: number;
  path: string;
  name: string;
  isDownloaded: boolean;
  progress: number;
  downloadedMediafiles: number;
  mediafiles: number;
  dateUpdate: Date;
  dateCreate: Date;
  isReachable: boolean;
  duplicateId?: number;

  constructor(data: any) {
    this.id = data.id;
    this.path = data.path;
    this.name = data.name;
    this.isDownloaded = Boolean(data.is_downloaded);
    this.progress = data.progress;
    this.downloadedMediafiles = data.downloaded_mediafiles;
    this.mediafiles = data.mediafiles;
    this.dateUpdate = new Date(data.date_update);
    this.dateCreate = new Date(data.date_create);
    this.isReachable = Boolean(data.is_reachable);
    this.duplicateId = data.duplicate_id;
  }
}
