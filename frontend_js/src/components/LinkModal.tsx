import { observer } from 'mobx-react';
import React from 'react';
import { NotificationManager } from 'react-notifications';
import { deleteOne, tagUnreachable } from '../services/links';
import { links } from '../states/links';

const LinkModal = () => {
  if (!links.editModal) {
    return null;
  }
  const { downloadedMediafiles, id, isDownloaded, path, name, progress, mediafiles, isReachable } = links.editModal;
  const close = (e) => {
    if (e.key === 'Escape') {
      links.openEdit(null);
    }
  };
  return (
    <div
      className="modal fade show bd-example-modal-lg"
      tabIndex={-1}
      role="dialog"
      style={{ display: 'block' }}
      onKeyDown={close}
    >
      <div className="modal-dialog" role="document">
        <div className="modal-content">
          <div className="modal-header">
            <h5 className="modal-title">{name}</h5>
            <button
              type="button"
              className="btn"
              data-dismiss="modal"
              aria-label="Close"
              onClick={() => links.openEdit(null)}
            >
              <span aria-hidden="true">X</span>
            </button>
          </div>
          <div className="modal-body">
            <table className="table">
              <tbody>
                <tr>
                  <td scope="col">id</td>
                  <td scope="row">{id}</td>
                </tr>
                <tr>
                  <td scope="col">path</td>
                  <td>
                    <a href={path}>{path}</a>
                  </td>
                </tr>
                <tr>
                  <td scope="col">name</td>
                  <td>{name}</td>
                </tr>
                <tr>
                  <td scope="col">is downloaded</td>
                  <td>
                    <input type="checkbox" disabled checked={isDownloaded} className="form-check-input" />
                  </td>
                </tr>
                <tr>
                  <td>is reachable</td>
                  <td>
                    <input
                      type="checkbox"
                      checked={isReachable}
                      className="form-check-input"
                      onChange={(e) => ontagUnreachable(id, e.target.checked)}
                    />
                  </td>
                </tr>
                <tr>
                  <td scope="col">progress</td>
                  <td>
                    <div className="progress">
                      <div
                        className="progress-bar"
                        role="progressbar"
                        style={{ width: progress + '%' }}
                        aria-valuenow={progress}
                        aria-valuemin={0}
                        aria-valuemax={100}
                      ></div>
                    </div>
                  </td>
                </tr>
                <tr>
                  <td scope="col">downloaded/all</td>
                  <td>
                    {downloadedMediafiles}/{mediafiles}
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
          <div className="modal-footer">
            <button type="button" className="btn btn-primary" onClick={() => links.checkDownloaded(id)}>
              check
            </button>
            <button type="button" className="btn btn-primary" onClick={() => links.downLoad(id)}>
              download
            </button>
            <button type="button" className="btn btn-primary" onClick={() => links.scanFilesForLink(id)}>
              scan
            </button>
            <button type="button" className="btn btn-danger" onClick={() => onDelete(id)}>
              delete
            </button>
            <button
              type="button"
              className="btn btn-secondary"
              data-dismiss="modal"
              onClick={() => links.openEdit(null)}
            >
              Close
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};

const onDelete = async (id: number) => {
  if (confirm(`delete ${id} ?`)) {
    const result = await deleteOne(id);
    alert(result.message);
    links.getAll();
  }
};

const ontagUnreachable = async (id: number, isReachable: boolean) => {
  if (confirm(`tag as ${isReachable ? '' : 'un'}reachable ${id} ?`)) {
    const result = await tagUnreachable(id, isReachable);
    NotificationManager.info(result.message);
    links.getAll();
    links.openEdit(null);
  }
};

export default observer(LinkModal);
