// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js

// https://github.com/samuelgoto/idle-detection

'use strict';

idl_test(
    ['idle-detection.tentative'],
    ['dom', 'html'],
    async (idl_array, t) => {
      await test_driver.set_permission({ name: 'idle-detection' }, 'granted', false);

      self.idle = new IdleDetector();
      let watcher = new EventWatcher(t, self.idle, ["change"]);
      let initial_state = watcher.wait_for("change");
      await self.idle.start();
      await initial_state;

      idl_array.add_objects({
        IdleDetector: ['idle'],
      });
    }
);
