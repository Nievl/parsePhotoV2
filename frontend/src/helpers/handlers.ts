import { AxiosError, AxiosPromise } from 'axios';
import { NotificationManager } from 'react-notifications';
import { iResult } from '../models/common';

export const HandleRequest = <Result>(
  promise: AxiosPromise<Result>,
  errorHandler?: (e: AxiosError) => any,
): Promise<Result> => {
  return promise
    .then(({ data }) => {
      if ((data as iResult)?.success) {
        NotificationManager.info((data as iResult)?.message);
      }
      return data;
    })
    .catch(errorHandler ?? defaultErrorHandler);
};

const defaultErrorHandler = (err: AxiosError<{ message?: string }>): void => {
  let message = 'Server is not responding';
  if (err.response) {
    if (typeof err.response.data === 'string') {
      message = err.response.data;
    } else {
      message = err.response.data.message;
    }
  }

  NotificationManager.error(message);
};
