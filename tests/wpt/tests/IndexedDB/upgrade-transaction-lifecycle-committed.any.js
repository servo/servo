// META: title=IndexedDB: committed versionchange transaction lifecycle
// META: global=window,worker
// META: script=resources/support.js
// META: script=resources/support-promises.js

// Spec: "https://w3c.github.io/IndexedDB/#upgrade-transaction-steps"
// "https://w3c.github.io/IndexedDB/#dom-idbdatabase-createobjectstore"
// "https://w3c.github.io/IndexedDB/#dom-idbdatabase-deleteobjectstore"

'use strict';

promise_test(t => {
    return createDatabase(t, database => {
        createBooksStore(t, database);
    }).then(database => {
        database.close();
    }).then(() => migrateDatabase(t, 2, (database, transaction, request) => {
        return new Promise((resolve, reject) => {
            transaction.addEventListener('complete', () => {
                resolve(new Promise((resolve, reject) => {
                    assert_equals(
                        request.transaction, transaction,
                        "The open request's transaction should be reset after " +
                        'oncomplete');
                    assert_throws_dom(
                        'InvalidStateError',
                        () => { database.createObjectStore('books2'); },
                        'createObjectStore exception should reflect that the ' +
                        'transaction is no longer running');
                    assert_throws_dom(
                        'InvalidStateError',
                        () => { database.deleteObjectStore('books'); },
                        'deleteObjectStore exception should reflect that the ' +
                        'transaction is no longer running');
                    resolve();
                }));
            }, false);
        });
    })).then(database => { database.close(); });
}, 'in the complete event handler for a committed transaction');

promise_test(t => {
    return createDatabase(t, database => {
        createBooksStore(t, database);
    }).then(database => {
        database.close();
    }).then(() => migrateDatabase(t, 2, (database, transaction, request) => {
        return new Promise((resolve, reject) => {
            transaction.addEventListener('complete', () => {
                setTimeout(() => {
                    resolve(new Promise((resolve, reject) => {
                        assert_equals(
                            request.transaction, null,
                            "The open request's transaction should be reset after " +
                            'oncomplete microtasks');
                        assert_throws_dom(
                            'InvalidStateError',
                            () => { database.createObjectStore('books2'); },
                            'createObjectStore exception should reflect that the ' +
                            'transaction is no longer running');
                        assert_throws_dom(
                            'InvalidStateError',
                            () => { database.deleteObjectStore('books'); },
                            'deleteObjectStore exception should reflect that the ' +
                            'transaction is no longer running');
                        resolve();
                    }));
                }, 0);
            }, false);
        });
    })).then(database => { database.close(); });
}, 'in a setTimeout(0) callback after the complete event is fired for a ' +
'committed transaction');