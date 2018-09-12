// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: global=window,dedicatedworker,sharedworker,serviceworker

'use strict';

promise_test(async t => {
  const srcs = ['./web-locks.idl', '/interfaces/html.idl'];
  const [weblocks, html] = await Promise.all(
    srcs.map(i => fetch(i).then(r => r.text())));

  const idl_array = new IdlArray();
  idl_array.add_idls(weblocks);
  idl_array.add_dependency_idls(html);

  try {
    await navigator.locks.request('name', l => { self.lock = l; });
  } catch (e) {
    // Surfaced in idlharness.js's test_object below.
  }

  idl_array.add_objects({
    LockManager: ['navigator.locks'],
    Lock: ['lock'],
  });

  if (self.Window) {
    idl_array.add_objects({ Navigator: ['navigator'] });
  } else {
    idl_array.add_objects({ WorkerNavigator: ['navigator'] });
  }

  idl_array.test();
});
