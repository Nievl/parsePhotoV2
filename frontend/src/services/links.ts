import axios from "axios";
import urlJoin from "url-join";
import { HandleRequest } from "../helpers/handlers";
import { iResult } from "../models/common";
import { iLink, iLinkCreateRequest } from "../models/links";

const path = "/links";

export function getAllLinks(
  isReachable: boolean,
  showDuplicate: boolean
): Promise<iLink[]> {
  return HandleRequest(
    axios.get<iLink[]>(urlJoin(path, ""), {
      params: { isReachable, showDuplicate },
    })
  );
}

export function createOne(link: iLinkCreateRequest): Promise<iResult> {
  try {
    return HandleRequest(axios.post<iResult>(urlJoin(path, ""), link));
  } catch (error) {
    console.log("error: ", error);
  }
}

export function deleteOne(id: number): Promise<iResult> {
  try {
    return HandleRequest(
      axios.delete<iResult>(urlJoin(path, ""), { params: { id } })
    );
  } catch (error) {
    console.log("error: ", error);
  }
}

export function downLoad(id: number): Promise<iResult> {
  try {
    return HandleRequest(
      axios.get<iResult>(urlJoin(path, "/download"), { params: { id } })
    );
  } catch (error) {
    console.log("error: ", error);
  }
}

export function checkDownloaded(id: number): Promise<iResult> {
  try {
    return HandleRequest(
      axios.get<iResult>(urlJoin(path, "/check_downloaded"), { params: { id } })
    );
  } catch (error) {
    console.log("error: ", error);
  }
}

export function tagUnreachable(
  id: number,
  isReachable: boolean
): Promise<iResult> {
  try {
    return HandleRequest(
      axios.get<iResult>(urlJoin(path, "/tag_unreachable"), {
        params: { id, isReachable },
      })
    );
  } catch (error) {
    console.log("error: ", error);
  }
}

export function scanFilesForLink(id: number): Promise<iResult> {
  try {
    return HandleRequest(
      axios.get<iResult>(urlJoin(path, "/scan_files_for_link"), {
        params: { id },
      })
    );
  } catch (error) {
    console.log("error: ", error);
  }
}

export function addDuplicate(
  linkId: number,
  duplicateId: number
): Promise<iResult> {
  try {
    return HandleRequest(
      axios.get<iResult>(urlJoin(path, "/add_duplicate"), {
        params: { linkId, duplicateId },
      })
    );
  } catch (error) {
    console.log("error: ", error);
  }
}
