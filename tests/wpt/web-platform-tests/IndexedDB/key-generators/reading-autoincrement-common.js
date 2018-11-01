// Returns the "name" property written to the object with the given ID.
function nameForId(id) {
  return `Object ${id}`;
}

// Initial database setup used by all the reading-autoincrement tests.
async function setupAutoincrementDatabase(testCase) {
  const database = await createDatabase(testCase, database => {
    const store = database.createObjectStore(
        'store', { autoIncrement: true, keyPath: 'id' });
    store.createIndex('by_name', 'name', { unique: true });
    store.createIndex('by_id', 'id', { unique: true });

    // Cover writing from the initial upgrade transaction.
    for (let i = 1; i <= 16; ++i) {
      if (i % 2 == 0) {
        store.put({name: nameForId(i), id: i});
      } else {
        store.put({name: nameForId(i)});
      }
    }
  });

  // Cover writing from a subsequent transaction.
  const transaction = database.transaction(['store'], 'readwrite');
  const store = transaction.objectStore('store');
  for (let i = 17; i <= 32; ++i) {
    if (i % 2 == 0) {
      store.put({name: nameForId(i), id: i});
    } else {
      store.put({name: nameForId(i)});
    }
  }
  await promiseForTransaction(testCase, transaction);

  return database;
}

// Returns the IDs used by the object store, sorted as strings.
//
// This is used to determine the correct order of records when retrieved from an
// index that uses stringified IDs.
function idsSortedByStringCompare() {
  const stringIds = [];
  for (let i = 1; i <= 32; ++i)
    stringIds.push(i);
  stringIds.sort((a, b) => indexedDB.cmp(`${a}`, `${b}`));
  return stringIds;
}

async function iterateCursor(testCase, cursorRequest, callback) {
  // This uses requestWatcher() directly instead of using promiseForRequest()
  // inside the loop to avoid creating multiple EventWatcher instances. In turn,
  // this avoids ending up with O(N) listeners for the request and O(N^2)
  // dispatched events.
  const eventWatcher = requestWatcher(testCase, cursorRequest);
  while (true) {
    const event = await eventWatcher.wait_for('success');
    const cursor = event.target.result;
    if (cursor === null)
      return;
    callback(cursor);
    cursor.continue();
  }
}

// Returns equivalent information to getAllKeys() by iterating a cursor.
//
// Returns an array with one dictionary per entry in the source. The dictionary
// has the properties "key" and "primaryKey".
async function getAllKeysViaCursor(testCase, cursorSource) {
  const results = [];
  await iterateCursor(testCase, cursorSource.openKeyCursor(), cursor => {
    results.push({ key: cursor.key, primaryKey: cursor.primaryKey });
  });
  return results;
}

// Returns equivalent information to getAll() by iterating a cursor.
//
// Returns an array with one dictionary per entry in the source. The dictionary
// has the properties "key", "primaryKey" and "value".
async function getAllViaCursor(testCase, cursorSource) {
  const results = [];
  await iterateCursor(testCase, cursorSource.openCursor(), cursor => {
    results.push({
      key: cursor.key,
      primaryKey: cursor.primaryKey,
      value: cursor.value,
    });
  });
  return results;
}