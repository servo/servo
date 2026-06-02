/**
 * @description - Function will create a database with the supplied name
 *                and also create an object store with the specified name.
 *                If a db with the name dbName exists, this will raze the
 *                existing DB beforehand.
 * @param {string} dbName
 * @param {string} objectStoreName
 * @param {testCase} t
 * @returns {Promise} - A promise that resolves to an indexedDB open request
 */
function createDB(dbName, objectStoreName, t) {
  return new Promise((resolve, reject) => {
    const openRequest = indexedDB.open(dbName);
    t.add_cleanup(() => {
      indexedDB.deleteDatabase(dbName);
    });
    openRequest.onerror = () => {
      reject(openRequest.error);
    };
    openRequest.onsuccess = () => {
      resolve(openRequest.result);
    };
    openRequest.onupgradeneeded = (event) => {
      openRequest.result.createObjectStore(objectStoreName);
    };
  });
}

/**
 * @description - This function will wrap an IDBTransaction in a promise,
 *                resolving in the oncomplete() method and rejecting with the
 *                transaction error in the onabort() case.
 * @param {IDBTransaction} transaction - The transaction to wrap in a promise.
 * @returns {Promise} - A promise that resolves when the transaction is either
 *                      aborted or completed.
 */
function transactionPromise(transaction) {
  return new Promise((resolve, reject) => {
    transaction.onabort = () => {
      reject(transaction.error);
    };
    transaction.oncomplete = () => {
      resolve();
    };
  });
}
