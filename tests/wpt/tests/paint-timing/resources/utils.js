function waitForAnimationFrames(count) {
  return new Promise(resolve => {
    if (count-- <= 0) {
      resolve();
    } else {
      requestAnimationFrame(() => {
        waitForAnimationFrames(count).then(resolve);
      });
    }
  });
}

// Asserts that there is currently no FCP reported. Pass t to add some wait, in case CSS is loaded
// and FCP is incorrectly fired afterwards.
async function assertNoFirstContentfulPaint(t) {
  await waitForAnimationFrames(3);
  assert_equals(performance.getEntriesByName('first-contentful-paint').length, 0, 'First contentful paint marked too early. ');
}

// Function that is resolved once FCP is reported, using PerformanceObserver. It rejects after a long
// wait time so that failing tests don't timeout.
async function assertFirstContentfulPaint(t) {
  return new Promise(resolve  => {
    function checkFCP() {
      const entries = performance.getEntriesByName('first-contentful-paint');
      if (entries.length === 1) {
        resolve(entries[0]);
      } else {
        t.step_timeout(checkFCP, 0);
      }
    }
    t.step(checkFCP);
  });
}

async function test_fcp(label, before_assert_fcp_func) {
  setup({"hide_test_state": true});
  const style = document.createElement('style');
  document.head.appendChild(style);
  await promise_test(async t => {
    assert_implements(window.PerformancePaintTiming, "Paint Timing isn't supported.");
    const main = document.getElementById('main');
    await new Promise(r => window.addEventListener('load', r));
    await assertNoFirstContentfulPaint(t);
    main.className = 'preFCP';
    await assertNoFirstContentfulPaint(t);
    const time_before_fcp_func = performance.now();
    if (before_assert_fcp_func) {
      await before_assert_fcp_func();
    }
    main.className = 'contentful';
    const entry = await assertFirstContentfulPaint(t);
    if ("paintTime" in entry) {
      if ("presentationTime" in entry) {
        assert_greater_than(entry.presentationTime, entry.paintTime);
        assert_equals(entry.startTime, entry.presentationTime);
      } else {
        assert_equals(entry.startTime, entry.paintTime);
      }
    }
  }, label);
}
