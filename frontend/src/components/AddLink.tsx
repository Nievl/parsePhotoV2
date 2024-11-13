import React, { useState } from "react";
import { NotificationManager } from "react-notifications";
import { createOne } from "../services/links";
import { links } from "../states/links";

export const AddLink = () => {
  const [path, usePath] = useState("");
  const onSubmit = async () => {
    if (path.trim().length === 0) {
      return;
    }
    if (!checkUrl(path)) {
      NotificationManager.error("path is incorrect");
      return;
    } else {
      await createOne({ path });
      links.getAll();
      usePath("");
    }
  };
  return (
    <div className="card card_new">
      <div className="card-body">
        <h5 className="card-title">Adding link</h5>
        <div className="input-group mb-3">
          <input
            type="text"
            id="link"
            onChange={(e) => usePath(e.target.value)}
            className="form-control"
            value={path}
          />
          <div className="input-group-append">
            <button
              className="btn btn-outline-primary"
              type="button"
              onClick={onSubmit}
            >
              Add
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};

const checkUrl = (url: string): string[] | null =>
  url.trim().match(/(http[s]?:\/\/[^\/\s]+\/)(.*)/);
