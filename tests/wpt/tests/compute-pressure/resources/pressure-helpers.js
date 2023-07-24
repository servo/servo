'use strict';

// These tests rely on the User Agent providing an implementation of
// platform compute pressure backends.
//
// In Chromium-based browsers this implementation is provided by a polyfill
// in order to reduce the amount of test-only code shipped to users. To enable
// these tests the browser must be run with these options:
//
//   --enable-blink-features=MojoJS,MojoJSTest

let mockPressureService = undefined;

function pressure_test(func, name, properties) {
  promise_test(async t => {
    if (mockPressureService === undefined) {
      if (isChromiumBased) {
        const mocks =
            await import('/resources/chromium/mock-pressure-service.js');
        mockPressureService = mocks.mockPressureService;
      }
    }
    assert_implements(
        mockPressureService,
        'missing mockPressureService after initialization');

    mockPressureService.start();

    t.add_cleanup(() => {
      mockPressureService.reset();
      return mockPressureService.stop();
    });
    return func(t, mockPressureService);
  }, name, properties);
}
