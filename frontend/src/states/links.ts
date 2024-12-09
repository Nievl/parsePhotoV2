import { makeAutoObservable } from 'mobx';
import { iLink } from '../models/links';
import { checkDownloaded, getAllLinks, downLoad, scanFilesForLink, addDuplicate } from '../services/links';
import { NotificationManager } from 'react-notifications';

class Links {
  links: iLink[] = [];
  editModal: iLink | null = null;

  constructor() {
    makeAutoObservable(this);
  }

  async getAll(isReachable: boolean = true, showDuplicate: boolean = false) {
    this.links = await getAllLinks(isReachable, showDuplicate);
  }

  openEdit(id: number | null) {
    if (id === null) {
      this.editModal = null;
    } else {
      this.editModal = this.links.find((l) => l.id === id) ?? null;
    }
  }

  async downLoad(id: number, refreshModal: boolean = true) {
    await downLoad(id);
    await this.getAll();
    if (refreshModal) {
      this.openEdit(id);
    }
  }

  async checkDownloaded(id: number) {
    await checkDownloaded(id);
    await this.getAll();
    this.openEdit(id);
  }

  async scanFilesForLink(id: number) {
    await scanFilesForLink(id);
  }

  async addDuplicate(linkId: number, duplicateId: number) {
    if (duplicateId !== 0) {
      const link = this.links.find((l) => l.id === duplicateId);
      if (!link) {
        NotificationManager.info(`link with ${duplicateId} not found`);
        return;
      }
    }

    await addDuplicate(linkId, duplicateId);
    await this.getAll(true, true);
    this.openEdit(linkId);
  }
}

export const links = new Links();
