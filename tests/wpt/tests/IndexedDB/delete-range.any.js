// META: title=Delete range
// META: script=resources/support.js

"use strict";

let entries = [
    { lower: 3, upper: 8, lowerOpen: false, upperOpen: false, expected: [1, 2, 9, 10]},
    { lower: 3, upper: 8, lowerOpen: true, upperOpen: false, expected: [1, 2, 3, 9, 10]},
    { lower: 3, upper: 8, lowerOpen: false, upperOpen: true, expected: [1, 2, 8, 9, 10]},
    { lower: 3, upper: 8, lowerOpen: true, upperOpen: true, expected: [1, 2, 3, 8, 9, 10]}
];

for (const entry of entries) {
    indexeddb_test(
        function upgrade_func(t, db) {
            db.createObjectStore("store");
        },
        function open_func(t, db) {
            const store = db.transaction("store", "readwrite", {durability: 'relaxed'}).objectStore("store");

            for (let i = 1; i <= 10; ++i) {
                store.put(i, i);
            }
            store.delete(IDBKeyRange.bound(entry.lower,
                entry.upper,
                entry.lowerOpen,
                entry.upperOpen));

            let keys = [];
            const cursor_request = store.openCursor();
            cursor_request.onsuccess = t.step_func(function () {
                const cursor = cursor_request.result;
                if (cursor) {
                    keys.push(cursor.key);
                    cursor.continue();
                } else {
                    assert_array_equals(entry.expected, keys, `Expected: ${entry.expected}, got: ${keys}.`);
                    t.done();
                }
            });
            cursor_request.onerror = t.unreached_func("Failed to open cursor for read request.");
        }
    )
}
