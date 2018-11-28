// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://wicg.github.io/media-capabilities/

'use strict';

promise_test(async () => {
  idl_test(
    ['media-capabilities'],
    ['html', 'cssom-view'],
    idl_array => {
      if (self.GLOBAL.isWorker()) {
        idl_array.add_objects({ WorkerNavigator: ['navigator'] });
      } else {
        idl_array.add_objects({ Navigator: ['navigator'] });
      }
      idl_array.add_objects({
        MediaCapabilities: ['navigator.mediaCapabilities'],
        Screen: ['screen'],
        ScreenLuminance: ['screen.luminance'],
      });
    }
  );
});
