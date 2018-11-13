// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

"use strict";

if (self.importScripts) {
  importScripts("/resources/testharness.js");
  importScripts("/resources/WebIDLParser.js", "/resources/idlharness.js");
}

// https://w3c.github.io/permissions/#idl-index

promise_test(async () => {
  const idl = await fetch("/interfaces/permissions.idl").then(r => r.text());
  const dom = await fetch("/interfaces/dom.idl").then(r => r.text());
  const html = await fetch("/interfaces/html.idl").then(r => r.text());

  const idl_array = new IdlArray();
  idl_array.add_idls(idl);
  idl_array.add_dependency_idls(dom);
  idl_array.add_dependency_idls(html);

  try {
    self.permissionStatus = await navigator.permissions.query({ name: "geolocation" });
  } catch (e) {
    // Will be surfaced in idlharness.js's test_object below.
  }

  if (self.GLOBAL.isWorker()) {
    idl_array.add_objects({ WorkerNavigator: ['navigator'] });
  } else {
    idl_array.add_objects({ Navigator: ['navigator'] });
  }

  idl_array.add_objects({
    Permissions: ['navigator.permissions'],
    PermissionStatus: ['permissionStatus']
  });
  idl_array.test();
}, "Test IDL implementation of Permissions API");
