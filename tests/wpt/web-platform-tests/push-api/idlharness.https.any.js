// META: global=window,worker
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/push-api/

promise_test(async () => {
  const srcs = [
    'push-api',
    'service-workers',
    'dom',
    'html'
  ];
  const [idl, worker, dom, html] = await Promise.all(
    srcs.map(i => fetch(`/interfaces/${i}.idl`).then(r => r.text())));

  const idl_array = new IdlArray();
  idl_array.add_idls(idl);
  idl_array.add_dependency_idls(worker);
  idl_array.add_dependency_idls(dom);
  idl_array.add_dependency_idls(html);
  idl_array.test();
}, 'push-api interfaces');
