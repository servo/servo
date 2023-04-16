// This mock provides a way to intercept renderer <-> browser mojo messages for
// navigator.subApps.* calls eliminating the need for an actual browser.
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

async function subapps_add_expect_reject_with_result(t, add_call_params, mocked_response, expected_results) {
  t.add_cleanup(async () => {
    await mockSubAppsService.reset();
    mockSubAppsService = null;
  });

  await createMockSubAppsService(Status.FAILURE, mocked_response, [], []);
  await navigator.subApps.add(add_call_params).then(
    result => {
      assert_unreached("Should have rejected: ", result);
    },
    error => {
      for (const app_id in expected_results) {
        assert_own_property(error, app_id, "Return results are missing entry for subapp.")
        assert_equals(error[app_id], expected_results[app_id], "Return results are not as expected.")
      }
    });
}

async function subapps_add_expect_success_with_result(t, add_call_params, mocked_response, expected_results) {
  t.add_cleanup(async () => {
    await mockSubAppsService.reset();
    mockSubAppsService = null;
  });

  await createMockSubAppsService(Status.SUCCESS, mocked_response, [], []);
  await navigator.subApps.add(add_call_params).then(result => {
    for (const app_id in expected_results) {
      assert_own_property(result, app_id, "Return results are missing entry for subapp.")
      assert_equals(result[app_id], expected_results[app_id], "Return results are not as expected.")
    }
  });
}

async function subapps_remove_expect_reject_with_result(t, remove_call_params, mocked_response, expected_results) {
  t.add_cleanup(async () => {
    await mockSubAppsService.reset();
    mockSubAppsService = null;
  });

  await createMockSubAppsService(Status.FAILURE, [], [], mocked_response);
  await navigator.subApps.remove(remove_call_params).then(
    result => {
      assert_unreached("Should have rejected: ", result);
    },
    error => {
      for (const app_id in expected_results) {
        assert_own_property(error, app_id, "Return results are missing entry for subapp.")
        assert_equals(error[app_id], expected_results[app_id], "Return results are not as expected.")
      }
    });
}

async function subapps_remove_expect_success_with_result(t, remove_call_params, mocked_response, expected_results) {
  t.add_cleanup(async () => {
    await mockSubAppsService.reset();
    mockSubAppsService = null;
  });

  await createMockSubAppsService(Status.SUCCESS, [], [], mocked_response);
  await navigator.subApps.remove(remove_call_params).then(result => {
    for (const app_id in expected_results) {
      assert_own_property(result, app_id, "Return results are missing entry for subapp.")
      assert_equals(result[app_id], expected_results[app_id], "Return results are not as expected.")
    }
  });
}
