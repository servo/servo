'use strict';

function openDatabase(name) {
    return new Promise((resolve, reject) => {
        const request = indexedDB.open(name, 1);
        request.onupgradeneeded = () => {
            request.result.createObjectStore('reads');
            request.result.createObjectStore('writes');
        };
        request.onsuccess = () => resolve(request.result);
        request.onerror = () => reject(request.error);
    });
}

function readForEver(database) {
    const step = () => {
        const transaction = database.transaction('reads', 'readonly');
        const request = transaction.objectStore('reads').get('k');
        request.onsuccess = () => setTimeout(step, 0);
        request.onerror = () => setTimeout(step, 0);
    };
    step();
}

function writeForEver(database) {
    let value = 0;
    const step = () => {
        const transaction = database.transaction('writes', 'readwrite');
        transaction.objectStore('writes').put(value++, 'k');
        transaction.oncomplete = () => setTimeout(step, 0);
        transaction.onabort = () => {};
        transaction.onerror = () => {};
    };
    step();
}

self.onmessage = async (event) => {
    const database = await openDatabase(event.data.name);
    readForEver(database);
    writeForEver(database);
    self.postMessage('ready');
};
