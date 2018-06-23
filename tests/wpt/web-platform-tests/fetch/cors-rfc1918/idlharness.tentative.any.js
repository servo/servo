// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

promise_test(async () => {
  const idl = await fetch('/interfaces/cors-rfc1918.idl').then(r => r.text());
  const html = await fetch('/interfaces/html.idl').then(r => r.text());
  const dom = await fetch('/interfaces/dom.idl').then(r => r.text());

  const idlArray = new IdlArray();
  idlArray.add_idls(idl);
  idlArray.add_dependency_idls(html);
  idlArray.add_dependency_idls(dom);

  const objects = {
    Document: ['document'],
    WorkerGlobalScope: ['self'],
  };
  idlArray.add_objects(objects);
  idlArray.test();
}, 'Test CORS RFC1918 interfaces');
