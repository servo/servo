// META: global=window,worker
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/performance-timeline/

'use strict';

promise_test(async t => {
  const observe = new Promise((resolve, reject) => {
    try {
      self.observer = new PerformanceObserver((entries, observer) => {
        self.entryList = entries;
        self.mark = entries.getEntries()[0];
        resolve();
      });
      observer.observe({ entryTypes: ['mark'] });
      performance.mark('test');
    } catch (e) {
      reject(e);
    }
  });
  const timeout = new Promise((_, reject) => {
    t.step_timeout(() => reject('Timed out waiting for observation'), 3000);
  });
  const user = await fetch('/interfaces/user-timing.idl').then(r => r.text());
  const execute_test = () => {
    idl_test(
      ['performance-timeline'],
      ['hr-time', 'dom'],
      idl_array => {
        idl_array.add_idls(user, {only: ['PerformanceMark']});
        idl_array.add_objects({
          Performance: ['performance'],
          // NOTE: PerformanceMark cascadingly tests PerformanceEntry
          PerformanceMark: ['mark'],
          PerformanceObserver: ['observer'],
          PerformanceObserverEntryList: ['entryList'],
        });
      }
    );
  };

  return Promise.race([observe, timeout]).then(
    execute_test,
    reason => {
      execute_test();
      return Promise.reject(reason);
    }
  );
})
