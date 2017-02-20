"use strict";

importScripts("/resources/testharness.js");
importScripts("/resources/WebIDLParser.js", "/resources/idlharness.js");

function doTest([untested, tested]) {
  var idlArray = new IdlArray();
  idlArray.add_untested_idls(untested);
  idlArray.add_idls(tested);

  idlArray.add_objects({
    WorkerNavigator: ['self.navigator'],
    WebSocket: ['new WebSocket("ws://foo")'],
    CloseEvent: ['new CloseEvent("close")'],
    Worker: [],
    MessageEvent: ['new MessageEvent("message", { data: 5 })'],
    DedicatedWorkerGlobalScope: ['self'],
  });

  idlArray.test();
};

function fetchData(url) {
  return fetch(url).then((response) => response.text());
}

promise_test(function() {
  return Promise.all([fetchData("resources/untested-interfaces.idl"),
                      fetchData("resources/interfaces.idl")])
                .then(doTest);
}, "Test driver");

done();
