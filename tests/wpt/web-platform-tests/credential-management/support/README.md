# CredentialManagement Testing

## OTPCredential Testing

In this test suite `otpcredential-helper.js` is a testing framework that enables
engines to test OTPCredential by intercepting the connection between the browser
and the underlying operating system and mock its behavior.

Usage:

1. Include `<script src="./support/otpcredential-helper.js"></script>` in your
test
2. Set expectations
```
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
