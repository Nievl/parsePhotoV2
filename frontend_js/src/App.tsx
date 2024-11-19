import React from 'react';
import { NotificationContainer } from 'react-notifications';
import { AddLink } from './components/AddLink';
import LinkModal from './components/LinkModal';
import TableLinks from './components/TableLinks';

function App() {
  return (
    <div className="container">
      <header>Gallery</header>
      <div className="body">Parse photo </div>
      <div className="card-deck mb-3 text-center">
        <AddLink />
      </div>
      <LinkModal />
      <TableLinks />
      <NotificationContainer />
    </div>
  );
}
export default App;
