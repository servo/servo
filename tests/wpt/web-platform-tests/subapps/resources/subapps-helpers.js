// This mock provides a way to intercept renderer <-> browser mojo messages for
// navigator.subApps.* calls eliminating the need for an actual browser.
//
// In Chromium-based browsers this implementation is provided by a polyfill
// in order to reduce the amount of test-only code shipped to users.

'use strict';

let mockSubAppsService = null;

const Status = {
  SUCCESS: 0,
  FAILURE: 1,
};

const AddCallResultCode = {
  SUCCESS_NEW_INSTALL: 0,
  SUCCESS_ALREADY_INSTALLED: 1,
  USER_INSTALL_DECLINED: 2,
  EXPECTED_APP_ID_CHECK_FAILED: 3,
  PARENT_APP_UNINSTALLED: 4,
  INSTALL_URL_INVALID: 5,
  NOT_VALID_MANIFEST_FOR_WEB_APP: 6,
  FAILURE: 7,
}

async function createMockSubAppsService(service_result_code, add_call_return_value, list_call_return_value) {
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
    mockSubAppsService.initialize(service_result_code, add_call_return_value, list_call_return_value);
  }
}

function subapps_test(func, description) {
  promise_test(async test => {
    test.add_cleanup(async () => {
      await mockSubAppsService.reset();
      mockSubAppsService = null;
    });
    await createMockSubAppsService(Status.SUCCESS, [], []);
    await func(test, mockSubAppsService);
  }, description);
}

async function subapps_add_expect_reject_with_result(t, add_call_params, mocked_response, expected_results) {
  t.add_cleanup(async () => {
    await mockSubAppsService.reset();
    mockSubAppsService = null;
  });

  await createMockSubAppsService(Status.FAILURE, mocked_response, []);
  await navigator.subApps.add(add_call_params)
    .then(result => {
      assert_unreached("Should have rejected.");
    })
    .catch(result => {
      for (const app_id in expected_results) {
        assert_own_property(result, app_id, "Return results are missing entry for subapp.")
        assert_equals(result[app_id], expected_results[app_id], "Return results are not as expected.")
      }
    });
}

async function subapps_add_expect_success_with_result(t, add_call_params, mocked_response, expected_results) {
  t.add_cleanup(async () => {
    await mockSubAppsService.reset();
    mockSubAppsService = null;
  });

  await createMockSubAppsService(Status.SUCCESS, mocked_response);
  await navigator.subApps.add(add_call_params)
    .catch(e => {
      assert_unreached("Should not have rejected.");
    })
    .then(result => {
      for (const app_id in expected_results) {
        assert_own_property(result, app_id, "Return results are missing entry for subapp.")
        assert_equals(result[app_id], expected_results[app_id], "Return results are not as expected.")
      }
    })
}
