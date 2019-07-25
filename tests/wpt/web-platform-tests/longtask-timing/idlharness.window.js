// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/longtasks/

'use strict';

idl_test(
  ['longtasks'],
  ['performance-timeline', 'hr-time'],
  (idl_array, t) => new Promise((resolve, reject) => {


    const longTask = () => {
      const begin = self.performance.now();
      while (self.performance.now() < begin + 100);
    }
    t.step_timeout(longTask, 0);

    const observer = new PerformanceObserver(entryList => {
      const entries = Array.from(entryList.getEntries());
      const attribution = entries.reduce(
          (sum, e) => sum.concat(e.attribution || []), []);
      idl_array.add_objects({
        PerformanceLongTaskTiming: entries,
        TaskAttributionTiming: attribution,
      });
      resolve();
    });
    observer.observe({entryTypes: ['longtask']});

    t.step_timeout(() => {
      reject('longtask entry was not observed');
    }, 1000);
  })
);
