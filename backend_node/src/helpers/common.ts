import dayjs from 'dayjs';
import { iResult } from 'src/entities/common';

export function chunkArray(array: string[], chunkSize: number): string[][] {
  const results: string[][] = [];
  while (array.length) {
    results.push(array.splice(0, chunkSize));
  }
  return results;
}

export function dateConvert(date?: string): string {
  if (date) {
    return dayjs(date).format(`YYYY-MM-DD HH:mm:ss`).toString();
  } else {
    return dayjs().format(`YYYY-MM-DD HH:mm:ss`).toString();
  }
}

export const booleanParser = (source: string): boolean => JSON.parse(source.toLowerCase());

export const resultMaker = (message?: string): iResult => ({ success: true, message });

export const checkUrl = (url: string): string[] | null => url.trim().match(urlReg);

const urlReg = /(http[s]?:\/\/[^\/\s]+\/)(.*)/;
