// META: title=IndexedDB: terminating a worker with a readwrite and a readonly transaction in flight must not wedge the database
// META: timeout=long

'use strict';

const iterations = 50;
const patience = 3000;

function openDatabase(name) {
    return new Promise((resolve, reject) => {
        const request = indexedDB.open(name, 1);
        request.onupgradeneeded = () => {
            request.result.createObjectStore('reads');
            request.result.createObjectStore('writes');
        };
        request.onsuccess = () => resolve(request.result);
        request.onerror = () => reject(request.error);
        request.onblocked = () => reject(new Error('open blocked'));
    });
}

function probe(database) {
    return new Promise((resolve) => {
        const transaction = database.transaction('writes', 'readonly');
        transaction.objectStore('writes').get('k').onsuccess = () => resolve('ok');
        transaction.onabort = () => resolve('aborted');
        transaction.onerror = () => resolve('error');
    });
}

function within(work) {
    return Promise.race([work, new Promise((resolve) => setTimeout(() => resolve('wedged'), patience))]);
}

function startWorker(name) {
    const worker = new Worker('resources/transaction-inflight-worker-terminate-worker.js');
    const ready = new Promise((resolve, reject) => {
        worker.onmessage = () => resolve();
        worker.onerror = () => reject(new Error('worker failed to start'));
        setTimeout(() => reject(new Error('worker never became ready')), patience);
    });
    worker.postMessage({ name });
    return ready.then(() => worker);
}

async function runIteration(index) {
    const name = `transaction-inflight-worker-terminate-${Date.now()}-${index}`;
    const worker = await startWorker(name);

    await new Promise((resolve) => setTimeout(resolve, 5 + Math.floor(Math.random() * 40)));
    worker.terminate();

    const database = await within(openDatabase(name));
    if (database === 'wedged')
        return 'open never settled';

    const result = await within(probe(database));
    database.close();
    return result === 'wedged' ? 'transaction never settled' : 'ok';
}

promise_test(async () => {
    for (let index = 0; index < iterations; index++) {
        const result = await runIteration(index);
        assert_equals(result, 'ok', `iteration ${index}`);
    }
}, 'IndexedDB: terminating a worker with a readwrite and a readonly transaction in flight must not wedge the database');
