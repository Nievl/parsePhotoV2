export interface iLink extends iLinkCreateRequest, iLinkUpdateRequest {
  id: number;
  dateCreate: Date;
  dateUpdate: Date;
  isReachable: boolean;
  name: string;
}

export interface iLinkCreateRequest {
  path: string;
}

export interface iLinkUpdateRequest {
  isDownloaded: boolean;
  progress: number;
  mediafiles: number;
  downloadedMediafiles: number;
}
