// Tests scheduler context propagation when a blur is caused by a task in one
// context and observed in another, within the same task (depending on origin
// and site isolation).
function runFocusChangeTest(t, crossOrigin) {
  window.onload = () => {
    const iframe = document.createElement('iframe');
    let src = location.href.slice(0, location.href.lastIndexOf('/'))
        + '/resources/focus-change-test-subframe.html';
    if (crossOrigin) {
      src = src.replace('://', '://www1.')
    }
    iframe.src = src;
    iframe.onload = () => {
      // TAB to focus the first input.
      test_driver.send_keys(document.body, "\ue004");
      // TAB again to focus the iframe's input.
      test_driver.send_keys(document.body, "\ue004");
    }
    document.body.appendChild(iframe);
  }

  let count = 0;

  window.onmessage = t.step_func((e) => {
    if (e.data.status === 'focus') {
      ++count;
      // The scheduling state is set when running the scheduler.postTask() and
      // propagated to continuations descending from the callback.
      if (count == 1) {
        scheduler.postTask(() => { input.focus(); }, {priority: 'background'});
      } else {
        assert_equals(count, 2);
        scheduler.postTask(async () => {
          await Promise.resolve();
          input.focus();
        }, {priority: 'background'});
      }
    } else {
      assert_equals(e.data.status, 'done');
      // If the default priority task runs before the background priority
      // continuation, then the scheduling state was used for the continuation,
      // which should not be the case for either cross- or same-origin frames.
      assert_false(
          e.data.didRun,
          'Did not expect the continuation to use inherited signals');
      if (count == 1) {
        test_driver.send_keys(document.body, "\ue004");
      } else {
        assert_equals(count, 2);
        t.done();
      }
    }
  });
}
