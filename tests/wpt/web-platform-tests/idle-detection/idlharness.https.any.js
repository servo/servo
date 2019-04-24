// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://github.com/samuelgoto/idle-detection

'use strict';

promise_test(async (t) => {
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

  self.idle = new IdleDetector({threshold: 60});

  let watcher = new EventWatcher(t, self.idle, ["change"]);

  self.idle.start();

  await watcher.wait_for("change");

  idl_array.add_objects({
    IdleDetector: ['idle'],
    IdleState: ['idle.state']
  });

  idl_array.test();
}, 'Test IDL implementation of Idle Detection API');
