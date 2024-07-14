# FedCM Testing

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
