// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js

// https://github.com/samuelgoto/idle-detection

'use strict';

promise_test(async (t) => {
  await test_driver.set_permission({ name: 'notifications' }, 'granted', false);

  const srcs = ['./idle-detection.idl',
                '/interfaces/dom.idl',
                '/interfaces/html.idl'];

  const [idle, dom, html] = await Promise.all(
    srcs.map(i => fetch(i).then(r => r.text()))
  );

  const idl_array = new IdlArray();
  idl_array.add_idls(idle);
  idl_array.add_dependency_idls(dom);
  idl_array.add_dependency_idls(html);

  self.idle = new IdleDetector({threshold: 60000});
  let watcher = new EventWatcher(t, self.idle, ["change"]);
  let initial_state = watcher.wait_for("change");
  await self.idle.start();
  await initial_state;

  idl_array.add_objects({
    IdleDetector: ['idle'],
    IdleState: ['idle.state']
  });

  idl_array.test();
}, 'Test IDL implementation of Idle Detection API');
