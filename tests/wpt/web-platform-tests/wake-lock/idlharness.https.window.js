// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/wake-lock/

'use strict';

promise_test(async () => {
  const srcs = ['wake-lock', 'dom', 'html'];
  const [wakelock, dom, html] = await Promise.all(
    srcs.map(i => fetch(`/interfaces/${i}.idl`).then(r => r.text())));

  const idl_array = new IdlArray();
  idl_array.add_idls(wakelock);
  idl_array.add_dependency_idls(dom);
  idl_array.add_dependency_idls(html);

  try {
    window.wakelock = await navigator.getWakeLock("screen");
    window.request = window.wakelock.createRequest();
  } catch (e) {
    // Surfaced in idlharness.js's test_object below.
  }

  idl_array.add_objects({
    Navigator: ['navigator'],
    WakeLock: ['wakelock'],
    WakeLockRequest: ['request']
  });
  idl_array.test();
}, 'Test IDL implementation of WakeLock API');
