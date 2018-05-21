// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

"use strict";

// https://wicg.github.io/feature-policy/

var idlArray = new IdlArray();

function doTest(idl) {
  idlArray.add_untested_idls("interface HTMLIFrameElement {};");
  idlArray.add_idls(idl);
  idlArray.add_objects({
    HTMLIframeElement: ['document.createElement("iframe")'],
  })
  idlArray.test();
  done();
}

promise_test(function () {
  return fetch("/interfaces/feature-policy.idl").then(response => response.text())
    .then(doTest);
}, "Test interfaces");
