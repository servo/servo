// The testing infra for FedCM is loaded using mojo js shim. To enable these
// tests the browser must be run with these options:
//
//   --enable-blink-features=MojoJS,MojoJSTest

import { MockFederatedAuthRequest } from './fedcm-mock.js';

export function fedcm_test(test_func, name, exception, properties) {
  promise_test(async (t) => {
    assert_implements(navigator.credentials, 'missing navigator.credentials');
    const mock = new MockFederatedAuthRequest();
    try {
      await test_func(t, mock);
    } catch (e) {
      assert_equals(exception, e.message)
    } finally {
      await mock.reset();
    }
  }, name, properties);
}
