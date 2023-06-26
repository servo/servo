'use strict';

// These tests rely on the User Agent providing an implementation of
// platform battery status backends.
//
// In Chromium-based browsers this implementation is provided by a polyfill
// in order to reduce the amount of test-only code shipped to users. To enable
// these tests the browser must be run with these options:
//
//   --enable-blink-features=MojoJS,MojoJSTest

let mockBatteryMonitor = undefined;

function battery_status_test(func, name, properties) {
  promise_test(async t => {
    if (mockBatteryMonitor === undefined) {
      if (isChromiumBased) {
        const mocks =
            await import('/resources/chromium/mock-battery-monitor.js');
        mockBatteryMonitor = mocks.mockBatteryMonitor;
      }
    }
    assert_implements(
        mockBatteryMonitor, 'missing mockBatteryMonitor after initialization');

    mockBatteryMonitor.start();

    t.add_cleanup(() => {
      mockBatteryMonitor.reset();
      return mockBatteryMonitor.stop();
    });
    return func(t, mockBatteryMonitor);
  }, name, properties);
}
