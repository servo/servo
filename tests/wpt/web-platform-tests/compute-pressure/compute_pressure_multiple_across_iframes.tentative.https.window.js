'use strict';

promise_test(async t => {
  const changes1_promise = new Promise((resolve, reject) => {
    const observer = new PressureObserver(resolve, {sampleRate: 1.0});
    t.add_cleanup(() => observer.disconnect());
    observer.observe('cpu').catch(reject);
  });

  // iframe numbers are aligned with observer numbers. The first observer is
  // in the main frame, so there is no iframe1.
  const iframe2 = document.createElement('iframe');
  document.body.appendChild(iframe2);

  const changes2_promise = new Promise((resolve, reject) => {
    const observer =
        new iframe2.contentWindow.PressureObserver(resolve, {sampleRate: 1.0});
    t.add_cleanup(() => observer.disconnect());
    observer.observe('cpu').catch(reject);
  });

  const iframe3 = document.createElement('iframe');
  document.body.appendChild(iframe3);

  const changes3_promise = new Promise((resolve, reject) => {
    const observer =
        new iframe3.contentWindow.PressureObserver(resolve, {sampleRate: 1.0});
    t.add_cleanup(() => observer.disconnect());
    observer.observe('cpu').catch(reject);
  });

  const [changes1, changes2, changes3] =
      await Promise.all([changes1_promise, changes2_promise, changes3_promise]);

  for (const changes of [changes1, changes2, changes3]) {
    assert_in_array(
        changes[0].state, ['nominal', 'fair', 'serious', 'critical'],
        'cpu pressure state');
  }
}, 'Three PressureObserver instances, in different iframes, receive changes');
