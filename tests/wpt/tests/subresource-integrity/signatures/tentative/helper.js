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

// As above in kValidKeys, but in JWK format (including the private key).
const kValidKeysJWK = {
  // https://www.rfc-editor.org/rfc/rfc9421.html#name-example-ed25519-test-key
  rfc: {
    "kty": "OKP",
    "crv": "Ed25519",
    "kid": "test-key-ed25519",
    "d": "n4Ni-HpISpVObnQMW0wOhCKROaIKqKtW_2ZYb2p9KcU",
    "x": "JrQLj5P_89iXES9-vFgrIy29clF9CC_oPPsw3c5D0bs"
  },

  // Matching private key to arbitrary public key above.
  arbitrary: {
    "crv": "Ed25519",
    "d": "MTodZiTA9CBsuIvSfO679TThkG3b7ce6R3sq_CdyVp4",
    "ext": true,
    "kty": "OKP",
    "x": "xDnP380zcL4rJ76rXYjeHlfMyPZEOqpJYjsjEppbuXE"
  }
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
function resourceURL(data, server_origin) {
  counter++;
  data.type ??= "application/javascript";
  data.counter = counter;
  let params = new URLSearchParams(data);
  let result = new URL("/subresource-integrity/signatures/resource.py?" + params.toString(), server_origin ?? self.location.origin);
  return result.href;
}

// Given a signature base (actually an arbitrary string) and a key in JWK
// format, generates a base64-encoded Ed25519 signature. Only available over
// HTTPS.
async function signSignatureBase(signatureBase, privateKeyJWK) {
  assert_true(self.isSecureContext, "Signatures can only be generated in secure contexts.");
  const privateKey = await crypto.subtle.importKey(
    'jwk',
    privateKeyJWK,
    'Ed25519',
    true, // extractable
    ['sign']
  );

  const encoder = new TextEncoder();
  const messageBytes = encoder.encode(signatureBase);

  const signatureBytes = await crypto.subtle.sign(
    { name: 'Ed25519' },
    privateKey,
    messageBytes
  );

  return btoa(String.fromCharCode(...new Uint8Array(signatureBytes)));
}

function generate_fetch_test(request_data, options, expectation, description) {
  promise_test(test => {
    const url = resourceURL(request_data, options.origin);
    let fetch_options = {};
    if (options.mode) {
      fetch_options.mode = options.mode;
    } else if (options.origin) {
      fetch_options.mode = "cors";
    }
    if (options.integrity) {
      fetch_options.integrity = options.integrity;
    }

    let fetcher = fetch(url, fetch_options);
    if (expectation == EXPECT_LOADED) {
      return fetcher.then(r => {
        const expected_status = options.mode == "no-cors" ? 0 : (request_data.status ?? 200);
        assert_equals(r.status, expected_status, `Response status is ${expected_status}.`);

        // Verify `accept-signature`: if the invalid key is present, both a valid and invalid
        // key were set. If just the valid key is present, that's the only key we should see
        // in the header.
        if (options.integrity?.includes(`ed25519-${kInvalidKey}`)) {
          assert_equals(r.headers.get(kAcceptSignature),
                        `sig0=("unencoded-digest";sf);keyid="${kValidKeys['rfc']}";tag="ed25519-integrity", sig1=("unencoded-digest";sf);keyid="${kInvalidKey}";tag="ed25519-integrity"`,
                        "`accept-signature` was set.");
        } else if (options.integrity?.includes(`ed25519-${kValidKeys['rfc']}`)) {
          assert_equals(r.headers.get(kAcceptSignature),
                        `sig0=("unencoded-digest";sf);keyid="${kValidKeys['rfc']}";tag="ed25519-integrity"`,
                        "`accept-signature` was set.");
        }
      });
    } else {
      return promise_rejects_js(test, TypeError, fetcher);
    }
  }, "`fetch()`: " + description);
}

function generate_query_test(query, options, expectation, description) {
  promise_test(test => {
    let url = new URL("/subresource-integrity/signatures/query-resource.py?" + query, self.location);

    let fetch_options = {};
    if (options.mode) {
      fetch_options.mode = options.mode;
    }
    if (options.integrity) {
      fetch_options.integrity = options.integrity;
    }

    let fetcher = fetch(url, fetch_options);
    if (expectation == EXPECT_LOADED) {
      return fetcher.then(r => {
        const expected_status = options.mode == "no-cors" ? 0 : 200;
        assert_equals(r.status, expected_status, `Response status is ${expected_status}.`);
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
