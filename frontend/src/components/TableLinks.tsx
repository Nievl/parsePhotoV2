import { SortingState, createColumnHelper, flexRender, getCoreRowModel, getSortedRowModel, useReactTable } from '@tanstack/react-table';
import { observer } from 'mobx-react';
import React, { useEffect } from 'react';
import { iLink } from '../models/links';
import { links } from '../states/links';

const columnHelper = createColumnHelper<iLink>();

const columns = [
  columnHelper.accessor('id', { cell: (info) => info.getValue() }),
  columnHelper.accessor('name', {
    header: () => 'Name',
    cell: (info) => info.getValue(),
  }),
  columnHelper.accessor('isDownloaded', {
    header: () => 'is downloaded',
    cell: (info) => <input type="checkbox" disabled checked={info.getValue()} className="form-check-input" />,
  }),
  columnHelper.accessor('progress', {
    header: () => 'progress',
    cell: (info) => (
      <div className="progress">
        <div
          className="progress-bar"
          role="progressbar"
          style={{ width: info.getValue() + '%' }}
          aria-valuenow={info.getValue()}
          aria-valuemin={0}
          aria-valuemax={100}
        />
      </div>
    ),
  }),
  columnHelper.accessor((row) => `${row.downloadedMediafiles}/${row.mediafiles}`, {
    id: 'downloaded/all',
    header: () => 'downloaded/all',
    cell: (info) => info.getValue(),
  }),
  columnHelper.accessor((row) => row.id, {
    id: 'actions',
    header: () => 'Actions',
    cell: (info) => (
      <button onClick={() => links.openEdit(info.getValue())} className="btn btn-primary m-1">
        <img src="/static/svg/Pencil.svg" className="icon" />
      </button>
    ),
  }),
  columnHelper.accessor('dateCreate', { cell: (info) => info.getValue() }),
  columnHelper.accessor('dateUpdate', { cell: (info) => info.getValue() }),
];

const TableLinks = () => {
  const [sorting, setSorting] = React.useState<SortingState>([]);
  useEffect(() => {
    links.getAll();
  }, []);

  const onGetList = (e) => {
    const { value } = e.target;
    if (value === 'all') {
      links.getAll();
    } else if (value === 'duplicates') {
      links.getAll(true, true);
    } else {
      links.getAll(false);
    }
  };

  const table = useReactTable({
    data: links.links ?? [],
    columns,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    state: {
      sorting,
    },
    onSortingChange: setSorting,
  });

  return (
    <div className="table_links">
      <h3>List of links</h3>
      <div className="inline">
        <div>total links: {links.links?.length ?? 0}</div>
        Get list
        <div>
          <input type="radio" value="all" name="getList" defaultChecked onChange={onGetList} /> all
          <input type="radio" value="archived" name="getList" onChange={onGetList} /> archived
          <input type="radio" value="duplicates" name="getList" onChange={onGetList} /> duplicates
        </div>
      </div>
      <div className="table_links_table">
        <table className="table table-striped">
          <thead>
            {table.getHeaderGroups().map((headerGroup) => (
              <tr key={headerGroup.id}>
                {headerGroup.headers.map((header) => {
                  return (
                    <th key={header.id} colSpan={header.colSpan}>
                      {header.isPlaceholder ? null : (
                        <div
                          {...{
                            className: header.column.getCanSort() ? 'cursor-pointer select-none' : '',
                            onClick: header.column.getToggleSortingHandler(),
                          }}
                        >
                          {flexRender(header.column.columnDef.header, header.getContext())}
                          {{
                            asc: ' 🔼',
                            desc: ' 🔽',
                          }[header.column.getIsSorted() as string] ?? null}
                        </div>
                      )}
                    </th>
                  );
                })}
              </tr>
            ))}
          </thead>
          <tbody>
            {table.getRowModel().rows.map((row) => (
              <tr key={row.id}>
                {row.getVisibleCells().map((cell) => (
                  <td key={cell.id}>{flexRender(cell.column.columnDef.cell, cell.getContext())}</td>
                ))}
              </tr>
            ))}
          </tbody>
          <tfoot>
            {table.getFooterGroups().map((footerGroup) => (
              <tr key={footerGroup.id}>
                {footerGroup.headers.map((header) => (
                  <th key={header.id}>{header.isPlaceholder ? null : flexRender(header.column.columnDef.footer, header.getContext())}</th>
                ))}
              </tr>
            ))}
          </tfoot>
        </table>
      </div>
    </div>
  );
};

export default observer(TableLinks);
