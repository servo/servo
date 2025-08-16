// META: global=window,worker
// META: title=IndexedDB: assure no crash when populating index
// META: script=../resources/support.js
// See https://crbug.com/434115938 for additional context and credits.

'use_strict';

promise_test(async t => {
  const db = (await new Promise(resolve => {
    const request = self.indexedDB.open('db', 1);
    request.addEventListener('upgradeneeded', resolve, {once: true});
  })).target.result;
  const store = db.createObjectStore('store', {keyPath: 'a.b', autoIncrement: true});
  store.put({});
  const index = store.createIndex('index', 'keypath');
  db.close();
}, "Assure no crash when populating index");
