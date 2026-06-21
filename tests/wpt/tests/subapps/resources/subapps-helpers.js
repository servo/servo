// This mock provides a way to intercept renderer <-> browser mojo messages for
// window.subApps.* calls eliminating the need for an actual browser.
//
// In Chromium-based browsers this implementation is provided by a polyfill
// in order to reduce the amount of test-only code shipped to users.

'use strict';

let mockSubAppsService = null;

// TODO(crbug.com/1408101): Figure out how to export SubAppsServiceResult (and
// get rid of this).
const Status = {
  SUCCESS: 0,
  FAILURE: 1,
};

async function createMockSubAppsService(service_result_code, add_call_return_value, list_call_return_value, remove_call_return_value) {
  if (typeof SubAppsServiceTest === 'undefined') {
    // Load test-only API helpers.
    const script = document.createElement('script');
    script.src = '/resources/test-only-api.js';
    script.async = false;
    const p = new Promise((resolve, reject) => {
      script.onload = () => { resolve(); };
      script.onerror = e => { reject(e); };
    })
    document.head.appendChild(script);
    await p;

    if (isChromiumBased) {
      // Chrome setup.
      await import('/resources/chromium/mock-subapps.js');
    } else {
      throw new Error('Unsupported browser.');
    }
  }
  assert_implements(SubAppsServiceTest, 'SubAppsServiceTest is not loaded properly.');

  if (mockSubAppsService === null) {
    mockSubAppsService = new SubAppsServiceTest();
    mockSubAppsService.initialize(service_result_code, add_call_return_value, list_call_return_value, remove_call_return_value);
  }
}

function subapps_test(func, description) {
  promise_test(async test => {
    test.add_cleanup(async () => {
      await mockSubAppsService.reset();
      mockSubAppsService = null;
    });
    await createMockSubAppsService(Status.SUCCESS, [], [], []);
    await func(test, mockSubAppsService);
  }, description);
}

async function subapps_add_expect_reject_with_result(
    t, add_call_params, mocked_response, expected_error_name) {
  t.add_cleanup(async () => {
    await mockSubAppsService.reset();
    mockSubAppsService = null;
  });

  await createMockSubAppsService(Status.FAILURE, mocked_response, [], []);
  await window.subApps.add(add_call_params)
      .then(
          result => {
            assert_unreached("Should have rejected: ", result);
          },
          error => {
            assert_true(error instanceof DOMException);
            assert_equals(error.name, expected_error_name);
          });
}

async function subapps_add_expect_success_with_result(t, add_call_params, mocked_response, expected_results) {
  t.add_cleanup(async () => {
    await mockSubAppsService.reset();
    mockSubAppsService = null;
  });

  await createMockSubAppsService(Status.SUCCESS, mocked_response, [], []);
  await window.subApps.add(add_call_params).then(result => {
    assert_equals(typeof result, 'object', 'add() should return an object');
    if (expected_results.installedApps) {
      for (const key in expected_results.installedApps) {
        assert_own_property(result.installedApps, key,
                            'installedApps should contain key');
        assert_equals(result.installedApps[key],
                      expected_results.installedApps[key]);
      }
      assert_equals(Object.keys(result.installedApps).length,
                    Object.keys(expected_results.installedApps).length);
    } else {
      assert_equals(Object.keys(result.installedApps).length, 0);
    }
    if (expected_results.failedApps) {
      for (const key in expected_results.failedApps) {
        assert_own_property(result.failedApps, key,
                            'failedApps should contain key');
        assert_true(result.failedApps[key] instanceof DOMException);
        assert_equals(result.failedApps[key].name,
                      expected_results.failedApps[key]);
      }
      assert_equals(Object.keys(result.failedApps).length,
                    Object.keys(expected_results.failedApps).length);
    } else {
      assert_equals(Object.keys(result.failedApps).length, 0);
    }
  });
}

async function subapps_remove_expect_reject_with_result(
    t, remove_call_params, mocked_response, expected_error_name) {
  t.add_cleanup(async () => {
    await mockSubAppsService.reset();
    mockSubAppsService = null;
  });

  await createMockSubAppsService(Status.FAILURE, [], [], mocked_response);
  await window.subApps.remove(remove_call_params)
      .then(
          result => {
            assert_unreached("Should have rejected: ", result);
          },
          error => {
            assert_true(error instanceof DOMException);
            assert_equals(error.name, expected_error_name);
          });
}

async function subapps_remove_expect_success_with_result(t, remove_call_params, mocked_response, expected_results) {
  t.add_cleanup(async () => {
    await mockSubAppsService.reset();
    mockSubAppsService = null;
  });

  await createMockSubAppsService(Status.SUCCESS, [], [], mocked_response);
  await window.subApps.remove(remove_call_params).then(result => {
    assert_equals(typeof result, 'object', 'remove() should return an object');
    if (expected_results.removedApps) {
      assert_array_equals(result.removedApps, expected_results.removedApps);
    } else {
      assert_equals(result.removedApps.length, 0);
    }
    if (expected_results.failedApps) {
      for (const key in expected_results.failedApps) {
        assert_own_property(result.failedApps, key,
                            'failedApps should contain key');
        assert_true(result.failedApps[key] instanceof DOMException);
        assert_equals(result.failedApps[key].name,
                      expected_results.failedApps[key]);
      }
      assert_equals(Object.keys(result.failedApps).length,
                    Object.keys(expected_results.failedApps).length);
    } else {
      assert_equals(Object.keys(result.failedApps).length, 0);
    }
  });
}
