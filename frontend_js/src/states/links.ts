import { makeAutoObservable } from "mobx";
import { iLink } from "../models/links";
import {
  checkDownloaded,
  getAllLinks,
  downLoad,
  scanFilesForLink,
} from "../services/links";

class Links {
  links: iLink[] = [];
  editModal: iLink | null = null;

  constructor() {
    makeAutoObservable(this);
  }

  async getAll(isReachable: boolean = true) {
    this.links = await getAllLinks(isReachable);
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
}

export const links = new Links();
