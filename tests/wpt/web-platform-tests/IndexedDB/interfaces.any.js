// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

promise_test(async t => {
  const srcs = ['html', 'dom', 'IndexedDB'];
  const [html, dom, indexeddb] = await Promise.all(
    srcs.map(i => fetch(`/interfaces/${i}.idl`).then(r => r.text())));

  const idl_array = new IdlArray();
  idl_array.add_idls(indexeddb);
  idl_array.add_dependency_idls(html);
  idl_array.add_dependency_idls(dom);
  idl_array.add_objects({
    IDBCursor: [],
    IDBCursorWithValue: [],
    IDBDatabase: [],
    IDBFactory: [self.indexedDB],
    IDBIndex: [],
    IDBKeyRange: [IDBKeyRange.only(0)],
    IDBObjectStore: [],
    IDBOpenDBRequest: [],
    IDBRequest: [],
    IDBTransaction: [],
    IDBVersionChangeEvent: ['new IDBVersionChangeEvent("type")'],
    DOMStringList: [],
  });

  idl_array.test();
}, 'Test driver');
