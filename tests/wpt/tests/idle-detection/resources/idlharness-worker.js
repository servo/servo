'use strict';

importScripts("/resources/testharness.js");
importScripts("/resources/WebIDLParser.js", "/resources/idlharness.js");

idl_test(
    ['idle-detection'],
    ['dom', 'html'],
    async (idl_array, t) => {
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

done();
