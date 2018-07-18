importScripts("/resources/testharness.js");
importScripts("/resources/WebIDLParser.js", "/resources/idlharness.js");

// https://drafts.css-houdini.org/worklets/

promise_test(async () => {
  const idl = await fetch('/interfaces/worklets.idl').then(r => r.text());

  const idl_array = new IdlArray();
  idl_array.add_idls(idl);
  idl_array.test();
}, 'worklets interfaces');
done();
