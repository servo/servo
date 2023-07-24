# CredentialManagement Testing

## OTPCredential Testing

In this test suite `otpcredential-helper.js` is a testing framework that enables
engines to test OTPCredential by intercepting the connection between the browser
and the underlying operating system and mock its behavior.

Usage:

1. Include the following in your test:
```html
<script src="/resources/test-only-api.js"></script>
<script src="support/otpcredential-helper.js"></script>
```
2. Set expectations
```javascript
await expect(receive).andReturn(() => {
  // mock behavior
})
```
3. Call `navigator.credentials.get({otp: {transport: ["sms"]}})`
4. Verify results

The mocking API is browser agnostic and is designed such that other engines
could implement it too.

Here are the symbols that are exposed to tests that need to be implemented
per engine:

- function receive(): the main/only function that can be mocked
- function expect(): the main/only function that enables us to mock it
- enum State {kSuccess, kTimeout}: allows you to mock success/failures

## FedCM Testing

`fedcm-mojojs-helper.js` exposes `fedcm_mojo_mock_test` which is a specialized
`promise_test` which comes pre-setup with the appropriate mocking infrastructure
to emulate platform federated auth backend. The mock is passed to the test
function as the second parameter.

Example usage:
```
<script type="module">
  import {fedcm_mojo_mock_test} from './support/fedcm-mojojs-helper.js';

  fedcm_mojo_mock_test(async (t, mock) => {
    mock.returnToken("https://idp.test/fedcm.json", "a_token");
    assert_equals("a_token", await navigator.credentials.get(options));
  }, "Successfully obtaining a token using mock.");
</script>
```

The chromium implementation uses the MojoJS shim.
