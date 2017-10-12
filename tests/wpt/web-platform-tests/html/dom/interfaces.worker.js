"use strict";

importScripts("/resources/testharness.js");
importScripts("/resources/WebIDLParser.js", "/resources/idlharness.js");

function doTest([html, dom, cssom, touchevents, uievents]) {
  var idlArray = new IdlArray();
  idlArray.add_untested_idls(dom + cssom + touchevents + uievents);
  idlArray.add_idls(html);

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
  return Promise.all([fetchData("/interfaces/html.idl"),
                      fetchData("/interfaces/dom.idl"),
                      fetchData("/interfaces/cssom.idl"),
                      fetchData("/interfaces/touchevents.idl"),
                      fetchData("/interfaces/uievents.idl")])
                .then(doTest);
}, "Test driver");

done();
