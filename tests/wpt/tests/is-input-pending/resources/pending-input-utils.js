// Dispatches the given sequence of actions and verifies isInputPending state
// after dispatch according to expectations. Returns when all dispatched input
// has been handled.
const pendingActionTest = async (label, target, actionCallback, expectations) => {
  promise_test(async () => {
    // Give focus to the page first, before running the test.
    await new test_driver.Actions()
      .pointerMove(0, 0)
      .pointerDown()
      .pointerUp()
      .send();

    // Register a handler to fetch the result of isInputPending from the target
    // window.
    const resultPromise = new Promise(res => {
      window.addEventListener('message', function handler(e) {
        if (e.data === 'check-input') return;
        res(e.data);
        window.removeEventListener('message', handler);
      });
    });

    // Signal to the target window to monitor isInputPending.
    target.postMessage('check-input', '*');

    const actions = actionCallback();
    const actionsPromise = actions.send();

    const {discrete, continuous} = await resultPromise;

    assert_equals(discrete, expectations.discrete, 'detected discrete input');
    assert_equals(continuous, expectations.continuous, 'detected continuous input');

    await actionsPromise;
  }, label);
}

const PendingInputUtils = {
  testDetectNoPendingInput(target, actionCallback, label) {
    pendingActionTest(label, target, actionCallback, {
      discrete: false,
      continuous: false,
    });
  },

  testDetectDiscretePendingInput(target, actionCallback, label) {
    pendingActionTest(label, target, actionCallback, {
      discrete: true,
      continuous: true,
    });
  },

  testDetectContinuousPendingInput(target, actionCallback, label) {
    pendingActionTest(label, target, actionCallback, {
      discrete: false,
      continuous: true,
    });
  },

  // Simulates a pointer event at the given coordinates, and tests that the
  // given target window cannot access it. Intended for cross-origin compliance
  // tests.
  testCannotAccessPendingInputAt(target, x, y, label) {
    PendingInputUtils.testDetectNoPendingInput(target, () => {
      return new test_driver.Actions().pointerMove(x, y).pointerDown().pointerUp();
    }, label);
  },
}
