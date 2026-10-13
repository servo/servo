// For each way of reading the record, reports which event fired and what reading the value threw.
function openDatabase(name) {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(name);
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });
}

function read(db, makeRequest, readValue) {
  return new Promise(resolve => {
    const store = db.transaction('store', 'readonly').objectStore('store');
    const request = makeRequest(store);
    request.onsuccess = () => {
      try {
        readValue(request.result);
        resolve({ event: 'success', exception: null });
      } catch (e) {
        resolve({ event: 'success', exception: e.name });
      }
    };
    request.onerror = e => {
      e.preventDefault();
      resolve({ event: 'error', error: request.error.name });
    };
  });
}

onmessage = async e => {
  const db = await openDatabase(e.data);
  const report = {
    get: await read(db, store => store.get('key'), result => result),
    getAll: await read(db, store => store.getAll(), result => result),
    cursor: await read(db, store => store.openCursor(), cursor => cursor.value),
    getAllRecords: 'getAllRecords' in IDBObjectStore.prototype
      ? await read(db, store => store.getAllRecords(), records => records[0].value)
      : 'unsupported',
  };
  db.close();
  postMessage(report);
};
