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
                        `sig0=("unencoded-digest";sf);keyid="${kInvalidKey}";tag="sri", sig1=("unencoded-digest";sf);keyid="${kValidKeys['rfc']}";tag="sri"`,
                        "`accept-signature` was set.");
        } else if (integrity.includes(`ed25519-${kValidKeys['rfc']}`)) {
          assert_equals(r.headers.get(kAcceptSignature),
                        `sig0=("unencoded-digest";sf);keyid="${kValidKeys['rfc']}";tag="sri"`,
                        "`accept-signature` was set.");
        }
      });
    } else {
      return promise_rejects_js(test, TypeError, fetcher);
    }
  }, "`fetch()`: " + description);
}

/*
 * Script tests
 *
 * Metadata for a script which expects to execute correctly and a script that
 * does not.
 */
const kScriptToExecute = {
  body: "window.hello = `world`;",
  hash: "PZJ+9CdAAIacg7wfUe4t/RkDQJVKM0mCZ2K7qiRhHFc=",

  signatures: {
    // ```
    // "unencoded-digest";sf: sha-256=:PZJ+9CdAAIacg7wfUe4t/RkDQJVKM0mCZ2K7qiRhHFc=:
    // "@signature-params": ("unencoded-digest";sf);keyid="JrQLj5P/89iXES9+vFgrIy29clF9CC/oPPsw3c5D0bs=";tag="sri"
    // ```
    rfc: "A1wOGCGrcfN34uMe2Umt7hJ6Su1MQFUL1QuT5nmk1R8I761eXUt2Zv4D5fOt1h1+4DlHPiA1FVwfJLbwlWnpBw==",

    // ```
    // "unencoded-digest";sf: sha-256=:PZJ+9CdAAIacg7wfUe4t/RkDQJVKM0mCZ2K7qiRhHFc=:
    // "@signature-params": ("unencoded-digest";sf);keyid="xDnP380zcL4rJ76rXYjeHlfMyPZEOqpJYjsjEppbuXE=";tag="sri"
    // ```
    arbitrary: "odk/ec9gO/DCcLPa1xSW1cSmB2s4XU3iDOxJAiod4v5/YBESjvwEJNAO9x4Frn/7rRIZ7sL5LwRNaymdHokOBQ=="
  }
};

const kScriptToBlock = {
  body: "assert_unreached(`This code should not execute.`);",
  hash: "FUSFR1N3vTmSGbI7q9jaMbHq+ogNeBfpznOIufaIfpc=",

  signatures: {
    // ```
    // "unencoded-digest";sf: sha-256=:FUSFR1N3vTmSGbI7q9jaMbHq+ogNeBfpznOIufaIfpc=:
    // "@signature-params": ("unencoded-digest";sf);keyid="JrQLj5P/89iXES9+vFgrIy29clF9CC/oPPsw3c5D0bs=";tag="sri"
    // ```
    rfc: "bR3fU6kzmMLol4GIcgj19+It0GB0dlKrD4ssH+SCz0vTLAdT3zt6Kfq4V60NnDdn62XGNr20b0TEKtfcremcDw==",

    // ```
    // "unencoded-digest";sf: sha-256=:FUSFR1N3vTmSGbI7q9jaMbHq+ogNeBfpznOIufaIfpc=:
    // "@signature-params": ("unencoded-digest";sf);keyid="xDnP380zcL4rJ76rXYjeHlfMyPZEOqpJYjsjEppbuXE";tag="sri"
    // ```
    arbitrary: "+5Iol+V65SW2qkpsTCyqYQJC4NZIsUGeNbO5kS9WdTboa9gg/nV6LwnySM02612YvPm++671nN9dBDJPYncuBA=="
  }
};

// These constants use the metadata above to create dictionaries that can be
// passed into `generate_script_test` below.
const kUnsignedShouldExecute = { body: kScriptToExecute['body'] };
const kUnsignedShouldBlock = { body: kScriptToBlock['body'] };
const kSignedShouldExecute = {
  body: kScriptToExecute['body'],
  digest: `sha-256=:${kScriptToExecute['hash']}:`,
  signatureInput: `signature=("unencoded-digest";sf);keyid="${kValidKeys['rfc']}";tag="sri"`,
  signature: `signature=:${kScriptToExecute['signatures']['rfc']}:`
};
const kSignedShouldBlock = {
  body: kScriptToBlock['body'],
  digest: `sha-256=:${kScriptToBlock['hash']}:`,
  signatureInput: `signature=("unencoded-digest";sf);keyid="${kValidKeys['rfc']}";tag="sri"`,
  signature: `signature=:${kScriptToBlock['signatures']['rfc']}:`
};
const kMultiplySignedShouldExecute = {
  body: kScriptToExecute['body'],
  digest: `sha-256=:${kScriptToExecute['hash']}:`,
  signatureInput: `signature1=("unencoded-digest";sf);keyid="${kValidKeys['rfc']}";tag="sri", ` +
                  `signature2=("unencoded-digest";sf);keyid="${kValidKeys['arbitrary']}";tag="sri"`,
  signature: `signature1=:${kScriptToExecute['signatures']['rfc']}:, ` +
             `signature2=:${kScriptToExecute['signatures']['arbitrary']}:`
};
const kMultiplySignedShouldBlock = {
  body: kScriptToBlock['body'],
  digest: `sha-256=:${kScriptToBlock['hash']}:`,
  signatureInput: `signature1=("unencoded-digest";sf);keyid="${kValidKeys['rfc']}";tag="sri", ` +
                  `signature2=("unencoded-digest";sf);keyid="${kValidKeys['arbitrary']}";tag="sri"`,
  signature: `signature1=:${kScriptToBlock['signatures']['rfc']}:, ` +
             `signature2=:${kScriptToBlock['signatures']['arbitrary']}:`
};

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
