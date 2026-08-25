browser.test.runTests([
  /**
   * Tests `browser.idle.queryState`.
   */
  async function testIdleQueryState() {
    const state = await browser.idle.queryState(60);
    browser.test.assertTrue(
        ['active', 'idle', 'locked'].includes(state),
        `Query state should return a valid state, got: ${state}`);
  },

  /**
   * Tests `browser.idle.setDetectionInterval`.
   */
  function testIdleSetDetectionInterval() {
    browser.idle.setDetectionInterval(30);
  },

  /**
   * Tests `browser.idle.onStateChanged` listener registration.
   * Note: There is currently no way to mock or force system idle states in WPT,
   * so we verify that listener registration and removal succeed without error.
   */
  function testIdleOnStateChanged() {
    const listener = (newState) => {};
    browser.idle.onStateChanged.addListener(listener);
    browser.idle.onStateChanged.removeListener(listener);
  },

  /**
   * Tests `browser.idle.queryState` error cases.
   */
  function testIdleQueryStateErrorCases() {
    // Verify it throws on invalid argument types.
    browser.test.assertThrows(() => browser.idle.queryState('invalid'));
    // Verify it throws when interval is less than the minimum (15).
    browser.test.assertThrows(() => browser.idle.queryState(10));
  },

  /**
   * Tests `browser.idle.setDetectionInterval` error cases.
   */
  function testIdleSetDetectionIntervalErrorCases() {
    // Verify it throws on invalid argument types.
    browser.test.assertThrows(() =>
                                  browser.idle.setDetectionInterval('invalid'));
    // Verify it throws when interval is less than the minimum (15).
    browser.test.assertThrows(() => browser.idle.setDetectionInterval(10));
  }
]);
