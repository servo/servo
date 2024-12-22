//
// Exciting constants we'll use for test cases below:
//
const kValidKeys = {
  // https://www.rfc-editor.org/rfc/rfc9421.html#name-example-ed25519-test-key
  rfc: "JrQLj5P/89iXES9+vFgrIy29clF9CC/oPPsw3c5D0bs=",

  // Randomly generated key:
  //
  // {
  //   "crv": "Ed25519",
  //   "d": "MTodZiTA9CBsuIvSfO679TThkG3b7ce6R3sq_CdyVp4",
  //   "ext": true,
  //   "kty": "OKP",
  //   "x": "xDnP380zcL4rJ76rXYjeHlfMyPZEOqpJYjsjEppbuXE"
  // }
  //
  arbitrary: "xDnP380zcL4rJ76rXYjeHlfMyPZEOqpJYjsjEppbuXE="
};

// A key with the right length that cannot be used to verify the HTTP response
// above.
const kInvalidKey = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

// Generated test expectations are more readable if we're using something
// other than a boolean.
const EXPECT_BLOCKED = "block";
const EXPECT_LOADED = "loaded";

const kAcceptSignature = "accept-signature";

// Given `{ digest: "...", body: "...", cors: true, type: "..." }`, generates
// the URL to a script resource that has the given characteristics.
let counter = 0;
function resourceURL(data) {
  counter++;
  data.type ??= "application/javascript";
  data.counter = counter;
  let params = new URLSearchParams(data);
  return "./resource.py?" + params.toString();
}

function generate_fetch_test(request_data, integrity, expectation, description) {
  promise_test(test => {
    const url = resourceURL(request_data);
    let options = {};
    if (integrity != "") {
      options.integrity = integrity;
    }

    let fetcher = fetch(url, options);
    if (expectation == EXPECT_LOADED) {
      return fetcher.then(r => {
        assert_equals(r.status, 200, "Response status is 200.");

        // Verify `accept-signature`: if the invalid key is present, both a valid and invalid
        // key were set. If just the valid key is present, that's the only key we should see
        // in the header.
        if (integrity.includes(`ed25519-${kInvalidKey}`)) {
          assert_equals(r.headers.get(kAcceptSignature),
                        `sig0=("identity-digest";sf);keyid="${kInvalidKey}";tag="sri", sig1=("identity-digest";sf);keyid="${kValidKeys['rfc']}";tag="sri"`,
                        "`accept-signature` was set.");
        } else if (integrity.includes(`ed25519-${kValidKeys['rfc']}`)) {
          assert_equals(r.headers.get(kAcceptSignature),
                        `sig0=("identity-digest";sf);keyid="${kValidKeys['rfc']}";tag="sri"`,
                        "`accept-signature` was set.");
        }
      });
    } else {
      return promise_rejects_js(test, TypeError, fetcher);
    }
  }, "`fetch()`: " + description);
}

function generate_script_test(request_data, integrity, expectation, description) {
  async_test(t => {
    let s = document.createElement('script');
    s.src = resourceURL(request_data);
    s.integrity = integrity;
    if (expectation == EXPECT_BLOCKED) {
      s.onerror = t.step_func_done(e => {
        assert_equals("error", e.type);
      });
      s.onload = t.unreached_func("Script should not execute.");
    } else {
      s.onload = t.step_func_done(e => {
        assert_equals("load", e.type);
      });
      s.onerror = t.unreached_func("Script should not fail.");
    }
    document.body.appendChild(s);
  }, "`<script>`: " + description);
}
