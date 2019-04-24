// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://w3c.github.io/paint-timing/

idl_test(
  ['paint-timing'],
  ['performance-timeline'],
  (idl_array, t) => {
    idl_array.add_objects({
      PerformancePaintTiming: ['paintTiming'],
    });

    const awaitPaint = new Promise(resolve => {
      let observer = new PerformanceObserver(list => {
        self.paintTiming = list.getEntries()[0];
        resolve();
      });
      observer.observe({ entryTypes: ['paint'] });
      const div = document.createElement('div');
      div.innerHTML = 'Hello World';
      document.body.appendChild(div);
    });
    const timeout = new Promise((_, reject) => {
      t.step_timeout(() => reject('Timed out waiting for paint event'), 3000);
    });
    return Promise.race([awaitPaint, timeout]);
  }
);
