import dayjs from 'dayjs';
import { iResult } from '../entities/common';

export function dateConvert(date?: string): string {
  if (date) {
    return dayjs(date).format(`YYYY-MM-DD HH:mm:ss`).toString();
  } else {
    return dayjs().format(`YYYY-MM-DD HH:mm:ss`).toString();
  }
}

export const booleanParser = (source: string): boolean => JSON.parse(source.toLowerCase());

export const resultMaker = (message?: string): iResult => ({ success: true, message });
