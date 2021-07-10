# Helper files:

- `resources/dispatcher.js` provides `send()` and `receive()`. This is an
  universal message passing API. It works cross-origin, and even access
  different browser context groups. Messages are queued, this means one doesn't
  need to wait for the receiver to listen, before sending the first message.

- `resources/executor.html` is a document. Test can send arbitrary javascript to evaluate
  in its execution context. This is universal and avoids introducing many
  specific `XXX-helper.html` resources. Moreover, tests are easier to read,
  because the whole logic of the test can be defined in a single file.

# Related documents:
- https://github.com/mikewest/credentiallessness/
- https://github.com/w3ctag/design-reviews/issues/582
