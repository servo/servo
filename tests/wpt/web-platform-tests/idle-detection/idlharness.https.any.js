// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://github.com/inexorabletash/idle-detection

'use strict';

promise_test(async () => {
  const srcs = ['./idle-detection.idl',
                '/interfaces/dom.idl',
                '/interfaces/html.idl'];
  const [idle, dom, html] = await Promise.all(
    srcs.map(i => fetch(i).then(r => r.text())));

  const idl_array = new IdlArray();
  idl_array.add_idls(idle);
  idl_array.add_dependency_idls(dom);
  idl_array.add_dependency_idls(html);

  self.idle = await navigator.idle.query();

  idl_array.add_objects({
    IdleManager: ['navigator.idle'],
    IdleStatus: ['idle'],
    IdleState: ['idle.state']
  });
  if (self.Window) {
    idl_array.add_objects({ Navigator: ['navigator'] });
  } else {
    idl_array.add_objects({ WorkerNavigator: ['navigator'] });
  }

  idl_array.test();
}, 'Test IDL implementation of Idle Detection API');
