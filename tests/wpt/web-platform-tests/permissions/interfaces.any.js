// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

"use strict";

if (self.importScripts) {
  importScripts("/resources/testharness.js");
  importScripts("/resources/WebIDLParser.js", "/resources/idlharness.js");
}

// https://w3c.github.io/permissions/#idl-index

promise_test(async () => {

  const permissions_idl = await fetch("/interfaces/permissions.idl")
      .then(response => response.text());
  const idl_array = new IdlArray();

  idl_array.add_untested_idls('interface Navigator {};');
  idl_array.add_untested_idls('[Exposed=(Window,Worker)] interface EventTarget {};');
  idl_array.add_untested_idls('interface EventHandler {};');
  idl_array.add_untested_idls('interface WorkerNavigator {};');
  idl_array.add_idls(permissions_idl);

  self.permissionStatus = await navigator.permissions.query({ name: "geolocation" });

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
