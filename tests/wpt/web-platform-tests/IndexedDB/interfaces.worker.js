"use strict";

importScripts("/resources/testharness.js");
importScripts("/resources/WebIDLParser.js", "/resources/idlharness.js");

async_test(function(t) {
  var request = new XMLHttpRequest();
  request.open("GET", "interfaces.idl");
  request.send();
  request.onload = t.step_func(function() {
    var idlArray = new IdlArray();
    var idls = request.responseText;

    // https://html.spec.whatwg.org/multipage/workers.html#workerglobalscope
    idlArray.add_untested_idls("[Exposed=Worker] interface WorkerGlobalScope {};");

    // https://html.spec.whatwg.org/multipage/webappapis.html#windoworworkerglobalscope-mixin
    idlArray.add_untested_idls(`[NoInterfaceObject, Exposed=(Window,Worker)]
                              interface WindowOrWorkerGlobalScope {};`);
    idlArray.add_untested_idls("WorkerGlobalScope implements WindowOrWorkerGlobalScope;");

    // https://dom.spec.whatwg.org/#interface-event
    idlArray.add_untested_idls("[Exposed=(Window,Worker)] interface Event { };");

    // https://dom.spec.whatwg.org/#interface-eventtarget
    idlArray.add_untested_idls("[Exposed=(Window,Worker)] interface EventTarget { };");

    // From Indexed DB:
    idlArray.add_idls(idls);

    idlArray.add_objects({
      IDBCursor: [],
      IDBCursorWithValue: [],
      IDBDatabase: [],
      IDBFactory: ["self.indexedDB"],
      IDBIndex: [],
      IDBKeyRange: ["IDBKeyRange.only(0)"],
      IDBObjectStore: [],
      IDBOpenDBRequest: [],
      IDBRequest: [],
      IDBTransaction: [],
      IDBVersionChangeEvent: ["new IDBVersionChangeEvent('foo')"],
      DOMStringList: [],
    });
    idlArray.test();
    t.done();
  });
});

done();
