function deleteThenOpen(dbName, upgradeFunc, bodyFunc) {
  const deleteRequest = indexedDB.deleteDatabase(dbName);
  deleteRequest.onerror = PerfTestRunner.logFatalError.bind('deleteDatabase should not fail');
  deleteRequest.onsuccess = (e) => {
    const openRequest = indexedDB.open(dbName);
    openRequest.onupgradeneeded = () => {
      upgradeFunc(openRequest.result, openRequest);
    }
    openRequest.onsuccess = () => {
      bodyFunc(openRequest.result, openRequest);
    }
    openRequest.onerror = (e) => {
      window.PerfTestRunner.logFatalError("Error setting up database " + dbName + ". Error: " + e.type);
    }
  }
}

// Non-performant on purpose - should cause relayouts.
function logToDocumentBody(stringOrStrings) {
  let div = document.createElement("div");
  document.body.appendChild(div);
  if (Array.isArray(stringOrStrings)) {
    for (let string of stringOrStrings) {
      div.innerHTML += string;
    }
  } else {
    div.innerHTML = stringOrStrings;
  }
  return div;
}

function createIncrementalBarrier(callback) {
  let count = 0;
  let called = false;
  return () => {
    if (called)
      PerfTestRunner.logFatalError("Barrier already used.");
    ++count;
    return () => {
      --count;
      if (count === 0) {
        if (called)
          PerfTestRunner.logFatalError("Barrier already used.");
        called = true;
        callback();
      }
    }
  }
}

/**
 * Retrieves a value from an IndexedDB object store.
 *
 * This function queries the object store for a given key and returns a promise
 * that resolves with the result of the get operation or rejects with an error
 * message.
 *
 * @param {IDBTransaction} transaction The IndexedDB transaction to perform the
 *     operation within.
 * @param {string} storeName The name of the object store to query.
 * @param {string|number} key The key used to retrieve the value from the store.
 * @return {Promise} A promise that resolves with the value from the store, or
 *     rejects with an error message.
 */
function getIDBValue(transaction, storeName, key) {
  return new Promise((resolve, reject) => {
    // Access the object store specified by storeName
    const objectStore = transaction.objectStore(storeName);
    const request = objectStore.get(key);

    // Resolve the promise with the result on successful retrieval
    request.onsuccess = () => resolve(request.result);

    // Reject the promise with an error message if the retrieval fails
    request.onerror = () => reject(`Failed to get '${key}' from '${
        storeName}' with error '${request.error}'`);
  });
}

/**
 * Retrieves multiple values from an IndexedDB object store.
 *
 * This function fetches data using an optional key range and batch size. If
 * both `range` and `batchSize` are omitted or null, it retrieves all records
 * from the store.
 *
 * @param {IDBTransaction} transaction The active IndexedDB transaction.
 * @param {string} storeName The name of the object store to query.
 * @param {IDBKeyRange|null} range Optional key range for filtering records.
 *     Defaults to null.
 * @param {number} batchSize Optional maximum number of records to retrieve. If
 *     omitted or 0, fetches all records.
 * @return {Promise<Array>} A promise that resolves with the retrieved records,
 *     or rejects on error.
 */
function getAllIDBValues(transaction, storeName, range = null, batchSize) {
  return new Promise((resolve, reject) => {
    const store = transaction.objectStore(storeName);
    const request = store.getAll(range, batchSize);

    // Triggered when the request completes successfully.
    request.onsuccess = () => resolve(request.result);

    // Triggered when an error occurs during the request.
    request.onerror = () => reject(`Failed to retrieve data from '${
        storeName}' with error '${request.error}'`);
  });
}

/**
 * Puts (inserts or updates) a value into an IndexedDB object store.
 *
 * This function inserts or updates the given value into the specified object
 * store. It returns a promise that resolves with the result of the put
 * operation or rejects with an error message if the operation fails.
 *
 * @param {IDBTransaction} transaction The IndexedDB transaction to perform the
 *     operation within.
 * @param {string} storeName The name of the object store to update.
 * @param {Object} value The value to insert or update in the store.
 * @return {Promise} A promise that resolves with the result of the put
 *     operation, or rejects with an error message
 */
function putIDBValue(transaction, storeName, value) {
  return new Promise((resolve, reject) => {
    // Access the object store specified by storeName
    const objectStore = transaction.objectStore(storeName);
    const request = objectStore.put(value);

    // Resolve the promise with the result on successful insertion or update
    request.onsuccess = () => resolve(request.result);

    // Reject the promise with an error message if the operation fails
    request.onerror = () => reject(
        `Failed to put value in '${storeName}' with error '${request.error}'`);
  });
}

/**
 * Retrieves multiple values from an IndexedDB object store by their keys.
 *
 * This function fetches records corresponding to the provided keys from the
 * given object store. If any request fails, the promise is rejected.
 *
 * @param {IDBObjectStore} store The object store to query.
 * @param {Array<IDBValidKey>} keys The keys to retrieve.
 * @return {Promise<Array<Object|undefined>>} Promise resolving to an array of
 *     retrieved values in the same order as input keys, or rejecting on
 * failure.
 */
function bulkGetIDBValues(store, keys) {
  return Promise.all(keys.map(
      key => new Promise((resolve, reject) => {
        const req = store.get(key);
        req.onsuccess = () => resolve(req.result);
        req.onerror = () => reject(
            `Failed to get key '${key}' from store with error '${req.error}'`);
      })));
}

/**
 * Inserts or updates multiple values in an IndexedDB object store.
 *
 * This function puts each value into the given object store and resolves when
 * all operations succeed.
 *
 * @param {IDBObjectStore} store The object store to update.
 * @param {Array<Object>} values The values to insert or update.
 * @return {Promise<void>} Promise that resolves on success or rejects on
 *     failure.
 */
function bulkPutIDBValues(store, values) {
  return Promise.all(values.map(
      value => new Promise((resolve, reject) => {
        const req = store.put(value);
        req.onsuccess = resolve;
        req.onerror = () =>
            reject(`Failed to put value in store with error '${req.error}'`);
      })));
}

/**
 * Deletes multiple entries from an IndexedDB object store by their keys.
 *
 * This function removes the records for the given keys from the object store.
 * If any delete operation fails, the promise is rejected.
 *
 * @param {IDBObjectStore} store The object store to delete from.
 * @param {Array<IDBValidKey>} keys The keys to delete.
 * @return {Promise<void>} Promise that resolves on success or rejects on
 *     failure.
 */
function bulkDeleteIDBValues(store, keys) {
  return Promise.all(keys.map(
      key => new Promise((resolve, reject) => {
        const req = store.delete(key);
        req.onsuccess = resolve;
        req.onerror = () =>
            reject(`Failed to delete key '${key}' from store: ${req.error}`);
      })));
}
function transactionCompletePromise(txn) {
  return new Promise((resolve, reject) => {
    txn.oncomplete = resolve;
    txn.onabort = reject;
  });
}

function reportDone() {
  window.parent.postMessage({
    message: "done"
  }, "*");
}

function reportError(event) {
  console.log(event);
  window.parent.postMessage({
    message: "error",
    data: event
  }, "*", );
}

if (window.PerfTestRunner) {
  // The file loaded here will signal a 'done' or 'error' message (see
  // reportDone or reportError) which signifies the end of a test run.
  window.PerfTestRunner.measurePageLoadTimeAfterDoneMessage = function(test) {

    let isDone = false;
    let outerDone = test.done;
    test.done = (done) => {
      isDone = true;
      if (outerDone)
        done();
    }

    test.run = () => {
      let file = PerfTestRunner.loadFile(test.path);

      let runOnce = function(finishedCallback) {
        let startTime;

        PerfTestRunner.logInfo("Testing " + file.length + " byte document.");

        let iframe = document.createElement("iframe");
        test.iframe = iframe;
        document.body.appendChild(iframe);

        iframe.sandbox = '';
        // Prevent external loads which could cause write() to return before
        // completing the parse.
        iframe.style.width = "600px";
        // Have a reasonable size so we're not line-breaking on every
        // character.
        iframe.style.height = "800px";
        iframe.contentDocument.open();

        let eventHandler = (event)=>{
          if (event.data.message == undefined) {
            console.log("Unknown message: ", event);
          } else if (event.data.message == "done") {
            PerfTestRunner.measureValueAsync(PerfTestRunner.now() - startTime);
            PerfTestRunner.addRunTestEndMarker();
            document.body.removeChild(test.iframe);
            finishedCallback();
          } else if (event.data.message == "error") {
            console.log("Error in page", event.data.data);
            PerfTestRunner.logFatalError("error in page: " + event.data.data.type);
          } else {
            console.log("Unknown message: ", event);
          }
          window.removeEventListener("message", eventHandler);
        }
        window.addEventListener("message", eventHandler, false);

        PerfTestRunner.addRunTestStartMarker();
        startTime = PerfTestRunner.now();

        if (test.params)
          iframe.contentWindow.params = test.params;

        iframe.contentDocument.write(file);
        PerfTestRunner.forceLayout(iframe.contentDocument);

        iframe.contentDocument.close();
      }

      let iterationCallback = () => {
        if (!isDone)
          runOnce(iterationCallback);
      }

      runOnce(iterationCallback);
    }

    PerfTestRunner.startMeasureValuesAsync(test)
  }
}
