"use strict";

importScripts("/resources/testharness.js");
importScripts("/resources/WebIDLParser.js", "/resources/idlharness.js");

var request = new XMLHttpRequest();
request.open("GET", "interfaces.idl");
request.send();
request.onload = function() {
  var idlArray = new IdlArray();
  var idls = request.responseText;

  idlArray.add_untested_idls("interface WorkerGlobalScope {};");
  idlArray.add_untested_idls("interface WorkerUtils {};");
  idlArray.add_untested_idls("WorkerGlobalScope implements WorkerUtils;");
  idlArray.add_untested_idls("interface Event { };");
  idlArray.add_untested_idls("interface EventTarget { };");

  // From Indexed DB:
  idlArray.add_idls("WorkerUtils implements IDBEnvironment;");
  idlArray.add_idls(idls);

  idlArray.add_objects({
    IDBCursor: [],
    IDBCursorWithValue: [],
    IDBDatabase: [],
    IDBEnvironment: [],
    IDBFactory: ["self.indexedDB"],
    IDBIndex: [],
    IDBKeyRange: ["IDBKeyRange.only(0)"],
    IDBObjectStore: [],
    IDBOpenDBRequest: [],
    IDBRequest: [],
    IDBTransaction: [],
    IDBVersionChangeEvent: ["new IDBVersionChangeEvent('foo')"],
  });
  idlArray.test();
  done();
};
