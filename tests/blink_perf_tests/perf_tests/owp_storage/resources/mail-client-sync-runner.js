try {
  const openRequest = window.indexedDB.open(databaseName);

  openRequest.onsuccess = async () => {
    const db = openRequest.result;

    const syncModule = getSyncModule(db, 'folder-1');

    let syncResponse;

    // Loop until there are no more changes to sync from server.
    // Syncs in batches to simulate realistic client-server delta sync behavior.
    while ((syncResponse = await syncModule.syncChangesFromServer()) !== null) {
      await syncModule.saveChangesToStore(syncResponse);
    }

    // reportDone() signals to the Perf test runner that measurements should
    // stop after the mailbox sync is complete
    reportDone();
  };
  openRequest.onerror = (event) => {
    reportError('Error opening database: ', event.target.error);
  };
} catch (error) {
  reportError('Error syncing mailbox: ', error);
}
