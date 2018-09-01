// META: title=StorageManager: estimate()

test(function(t) {
    assert_true(navigator.storage.estimate() instanceof Promise);
}, 'estimate() method returns a Promise');

promise_test(function(t) {
    return navigator.storage.estimate().then(function(result) {
        assert_true(typeof result === 'object');
        assert_true('usage' in result);
        assert_equals(typeof result.usage, 'number');
        assert_true('quota' in result);
        assert_equals(typeof result.quota, 'number');
    });
}, 'estimate() resolves to dictionary with members');

promise_test(function(t) {
    const large_value = new Uint8Array(1e6);
    const dbname = `db-${location}-${t.name}`;
    let db, before, after;

    indexedDB.deleteDatabase(dbname);
    return new Promise((resolve, reject) => {
            const open = indexedDB.open(dbname);
            open.onerror = () => { reject(open.error); };
            open.onupgradeneeded = () => {
                const connection = open.result;
                connection.createObjectStore('store');
            };
            open.onsuccess = () => {
                const connection = open.result;
                t.add_cleanup(() => {
                    connection.close();
                    indexedDB.deleteDatabase(dbname);
                });
                resolve(connection);
            };
        })
        .then(connection => {
            db = connection;
            return navigator.storage.estimate();
        })
        .then(estimate => {
            before = estimate.usage;
            return new Promise((resolve, reject) => {
                const tx = db.transaction('store', 'readwrite');
                tx.objectStore('store').put(large_value, 'key');
                tx.onabort = () => { reject(tx.error); };
                tx.oncomplete = () => { resolve(); };
            });
        })
        .then(() => {
            return navigator.storage.estimate();
        })
        .then(estimate => {
            after = estimate.usage;
            assert_greater_than(after, before,
                                'estimated usage should increase');
        });
}, 'estimate() shows usage increase after 1MB IndexedDB record is stored');
