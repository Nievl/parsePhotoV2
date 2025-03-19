export interface Mediafile extends CreateMediafileDto {
  id: number;
  dateAdded: string;
}

export interface CreateMediafileDto {
  name: string;
  path: string;
  hash: string;
  size: number;
  linkId: string;
}

export class MediafileDto implements Mediafile {
  id: number;
  dateAdded: string;
  name: string;
  path: string;
  hash: string;
  size: number;
  linkId: string;

  constructor(data: any) {
    this.id = data.mediafile_id;
    this.path = data.mediafile_path;
    this.name = data.mediafile_name;
    this.hash = data.mediafile_hash;
    this.size = data.mediafile_size;
    this.dateAdded = data.mediafile_date_added;
    this.linkId = data.mediafile_link_id;
  }
}
