// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

promise_test(async t => {
  const [html, dom, indexeddb] = await Promise.all([
    '/interfaces/html.idl',
    '/interfaces/dom.idl',
    '/interfaces/IndexedDB.idl',
  ].map(url => fetch(url).then(response => response.text())));

  const idl_array = new IdlArray();
  idl_array.add_untested_idls('interface LinkStyle {};');  // Needed by html
  idl_array.add_untested_idls(html);
  idl_array.add_untested_idls(dom);
  idl_array.add_idls(indexeddb);
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
    IDBVersionChangeEvent: [new IDBVersionChangeEvent('')],
    DOMStringList: [],
  });

  idl_array.test();
}, 'Test driver');
