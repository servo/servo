// Returns an IndexedDB database name likely to be unique to the test case.
const databaseName = (testCase) => {
    return 'db' + self.location.pathname + '-' + testCase.name;
};

// Creates an EventWatcher covering all the events that can be issued by
// IndexedDB requests and transactions.
const requestWatcher = (testCase, request) => {
    return new EventWatcher(testCase, request,
        ['error', 'success', 'upgradeneeded']);
};

// Migrates an IndexedDB database whose name is unique for the test case.
//
// newVersion must be greater than the database's current version.
//
// migrationCallback will be called during a versionchange transaction and will
// be given the created database and the versionchange transaction.
//
// Returns a promise. If the versionchange transaction goes through, the promise
// resolves to an IndexedDB database that must be closed by the caller. If the
// versionchange transaction is aborted, the promise resolves to an error.
const migrateDatabase = (testCase, newVersion, migrationCallback) => {
    // We cannot use eventWatcher.wait_for('upgradeneeded') here, because
    // the versionchange transaction auto-commits before the Promise's then
    // callback gets called.
    return new Promise((resolve, reject) => {
        const request = indexedDB.open(databaseName(testCase), newVersion);
        request.onupgradeneeded = testCase.step_func(event => {
            const database = event.target.result;
            const transaction = event.target.transaction;
            let abortCalled = false;

            // We wrap IDBTransaction.abort so we can set up the correct event
            // listeners and expectations if the test chooses to abort the
            // versionchange transaction.
            const transactionAbort = transaction.abort.bind(transaction);
            transaction.abort = () => {
                request.onerror = event => {
                    event.preventDefault();
                    resolve(event);
                };
                request.onsuccess = () => reject(new Error(
                    'indexedDB.open should not succeed after the ' +
                    'versionchange transaction is aborted'));
                transactionAbort();
                abortCalled = true;
            }

            migrationCallback(database, transaction);
            if (!abortCalled) {
                request.onsuccess = null;
                resolve(requestWatcher(testCase, request).wait_for('success'));
            }
        });
        request.onerror = event => reject(event.target.error);
        request.onsuccess = () => reject(new Error(
            'indexedDB.open should not succeed without creating a ' +
            'versionchange transaction'));
    }).then(event => event.target.result || event.target.error);
};

// Creates an IndexedDB database whose name is unique for the test case.
//
// setupCallback will be called during a versionchange transaction, and will be
// given the created database and the versionchange transaction.
//
// Returns a promise that resolves to an IndexedDB database. The caller must
// close the database.
const createDatabase = (testCase, setupCallback) => {
    const request = indexedDB.deleteDatabase(databaseName(testCase));
    const eventWatcher = requestWatcher(testCase, request);

    return eventWatcher.wait_for('success').then(event =>
        migrateDatabase(testCase, 1, setupCallback));
};

// Opens an IndexedDB database without performing schema changes.
//
// The given version number must match the database's current version.
//
// Returns a promise that resolves to an IndexedDB database. The caller must
// close the database.
const openDatabase = (testCase, version) => {
    const request = indexedDB.open(databaseName(testCase), version);
    const eventWatcher = requestWatcher(testCase, request);
    return eventWatcher.wait_for('success').then(
        event => event.target.result);
}

// The data in the 'books' object store records in the first example of the
// IndexedDB specification.
const BOOKS_RECORD_DATA = [
    { title: 'Quarry Memories', author: 'Fred', isbn: 123456 },
    { title: 'Water Buffaloes', author: 'Fred', isbn: 234567 },
    { title: 'Bedrock Nights', author: 'Barney', isbn: 345678 },
];

// Creates a 'books' object store whose contents closely resembles the first
// example in the IndexedDB specification.
const createBooksStore = (testCase, database) => {
    const store = database.createObjectStore('books',
        { keyPath: 'isbn', autoIncrement: true });
    store.createIndex('by_author', 'author');
    store.createIndex('by_title', 'title', { unique: true });
    for (let record of BOOKS_RECORD_DATA)
        store.put(record);
    return store;
};

// Creates a 'not_books' object store used to test renaming into existing or
// deleted store names.
const createNotBooksStore = (testCase, database) => {
    const store = database.createObjectStore('not_books');
    store.createIndex('not_by_author', 'author');
    store.createIndex('not_by_title', 'title', { unique: true });
    return store;
};

// Verifies that an object store's indexes match the indexes used to create the
// books store in the test database's version 1.
//
// The errorMessage is used if the assertions fail. It can state that the
// IndexedDB implementation being tested is incorrect, or that the testing code
// is using it incorrectly.
const checkStoreIndexes = (testCase, store, errorMessage) => {
    assert_array_equals(
        store.indexNames, ['by_author', 'by_title'], errorMessage);
    const authorIndex = store.index('by_author');
    const titleIndex = store.index('by_title');
    return Promise.all([
        checkAuthorIndexContents(testCase, authorIndex, errorMessage),
        checkTitleIndexContents(testCase, titleIndex, errorMessage),
    ]);
};

// Verifies that an object store's key generator is in the same state as the
// key generator created for the books store in the test database's version 1.
//
// The errorMessage is used if the assertions fail. It can state that the
// IndexedDB implementation being tested is incorrect, or that the testing code
// is using it incorrectly.
const checkStoreGenerator = (testCase, store, expectedKey, errorMessage) => {
    const request = store.put(
        { title: 'Bedrock Nights ' + expectedKey, author: 'Barney' });
    const eventWatcher = requestWatcher(testCase, request);
    return eventWatcher.wait_for('success').then(() => {
        const result = request.result;
        assert_equals(result, expectedKey, errorMessage);
    });
};

// Verifies that an object store's contents matches the contents used to create
// the books store in the test database's version 1.
//
// The errorMessage is used if the assertions fail. It can state that the
// IndexedDB implementation being tested is incorrect, or that the testing code
// is using it incorrectly.
const checkStoreContents = (testCase, store, errorMessage) => {
    const request = store.get(123456);
    const eventWatcher = requestWatcher(testCase, request);
    return eventWatcher.wait_for('success').then(() => {
        const result = request.result;
        assert_equals(result.isbn, BOOKS_RECORD_DATA[0].isbn, errorMessage);
        assert_equals(result.author, BOOKS_RECORD_DATA[0].author, errorMessage);
        assert_equals(result.title, BOOKS_RECORD_DATA[0].title, errorMessage);
    });
};

// Verifies that index matches the 'by_author' index used to create the
// by_author books store in the test database's version 1.
//
// The errorMessage is used if the assertions fail. It can state that the
// IndexedDB implementation being tested is incorrect, or that the testing code
// is using it incorrectly.
const checkAuthorIndexContents = (testCase, index, errorMessage) => {
    const request = index.get(BOOKS_RECORD_DATA[2].author);
    const eventWatcher = requestWatcher(testCase, request);
    return eventWatcher.wait_for('success').then(() => {
        const result = request.result;
        assert_equals(result.isbn, BOOKS_RECORD_DATA[2].isbn, errorMessage);
        assert_equals(result.title, BOOKS_RECORD_DATA[2].title, errorMessage);
    });
};

// Verifies that an index matches the 'by_title' index used to create the books
// store in the test database's version 1.
//
// The errorMessage is used if the assertions fail. It can state that the
// IndexedDB implementation being tested is incorrect, or that the testing code
// is using it incorrectly.
const checkTitleIndexContents = (testCase, index, errorMessage) => {
    const request = index.get(BOOKS_RECORD_DATA[2].title);
    const eventWatcher = requestWatcher(testCase, request);
    return eventWatcher.wait_for('success').then(() => {
        const result = request.result;
        assert_equals(result.isbn, BOOKS_RECORD_DATA[2].isbn, errorMessage);
        assert_equals(result.author, BOOKS_RECORD_DATA[2].author, errorMessage);
    });
};
