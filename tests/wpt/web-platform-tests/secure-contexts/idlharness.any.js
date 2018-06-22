// META: global=window,worker
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/webappsec-secure-contexts/

'use strict';

promise_test(async () => {
  const idl = await fetch("/interfaces/secure-contexts.idl").then(r => r.text());
  const workers = await fetch("/interfaces/dedicated-workers.idl").then(r => r.text());

  const idl_array = new IdlArray();
  idl_array.add_idls(idl);
  idl_array.add_dependency_idls(workers);
  idl_array.add_objects({
    WindowOrWorkerGlobalScope: ["self"],
  });
  idl_array.test();
}, "Test IDL implementation of Secure Contexts");
