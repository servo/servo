//
// Validate signature-based SRI's interaction between signed script responses
// and `<script integrity>` assertions.
//

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

const kScriptToExecute = {
  body: "window.hello = `world`;",
  hash: "PZJ+9CdAAIacg7wfUe4t/RkDQJVKM0mCZ2K7qiRhHFc=",

  signatures: {
    // ```
    // "identity-digest": sha-256=:PZJ+9CdAAIacg7wfUe4t/RkDQJVKM0mCZ2K7qiRhHFc=:
    // "@signature-params": ("identity-digest";sf);alg="ed25519";keyid="JrQLj5P/89iXES9+vFgrIy29clF9CC/oPPsw3c5D0bs=";tag="sri"
    // ```
    rfc: "SBTcEpLwiDpHvxOFkajwl+S9Mnwf+86JLyhdL1LoMaFbyaqKqdkOu/6/HyNmKdRJK59heDMaIut5/4IXahH/Ag==",

    // ```
    // "identity-digest": sha-256=:PZJ+9CdAAIacg7wfUe4t/RkDQJVKM0mCZ2K7qiRhHFc=:
    // "@signature-params": ("identity-digest";sf);alg="ed25519";keyid="xDnP380zcL4rJ76rXYjeHlfMyPZEOqpJYjsjEppbuXE=";tag="sri"
    // ```
    arbitrary: "EaC2ECm9TD+W5o1LATMd6YwKX+tfl2vZhe9mwKzmFwzHIPsKoegEYX9o/a/yQ0L/rIBWIKYTUaOSQ8Tig0s3Cw=="
  }
};

const kScriptToBlock = {
  body: "assert_unreached(`This code should not execute.`);",
  hash: "FUSFR1N3vTmSGbI7q9jaMbHq+ogNeBfpznOIufaIfpc=",

  signatures: {
    // ```
    // "identity-digest": sha-256=:FUSFR1N3vTmSGbI7q9jaMbHq+ogNeBfpznOIufaIfpc=:
    // "@signature-params": ("identity-digest";sf);alg="ed25519";keyid="JrQLj5P/89iXES9+vFgrIy29clF9CC/oPPsw3c5D0bs=";tag="sri"
    // ```
    rfc: "WE+KckOA+tcmoIlFZjBreg6uMrH7eRLHmioElLIiSaVINe+gyAwmvgWsJeoZdFQ7b92zJP3zWymikJsmKuAuAg==",

    // ```
    // "identity-digest": sha-256=:FUSFR1N3vTmSGbI7q9jaMbHq+ogNeBfpznOIufaIfpc=:
    // "@signature-params": ("identity-digest";sf);alg="ed25519";keyid="xDnP380zcL4rJ76rXYjeHlfMyPZEOqpJYjsjEppbuXE";tag="sri"
    // ```
    arbitrary: "R7yvyU8E+nOPB3JVOaGLtIBfldw/UCcFGWi4e7uV9KpWvXhFN0ISV/g6PXRzGFtmChobjND0PU7tgm0WyafjCQ=="
  }
};

//
// Equally exciting helper functions
//

// Given `{ digest: "...", body: "...", cors: true, type: "..." }`, generates
// the URL to a script resource that has the given characteristics.
let counter = 0;
function resourceURL(data) {
  counter++;
  data.type = "application/javascript";
  data.counter = counter;
  let params = new URLSearchParams(data);
  return "./resource.py?" + params.toString();
}

const EXPECT_BLOCKED = "block";
const EXPECT_LOADED = "loaded";

function generate_test(request_data, integrity, expectation, description) {
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
  }, description);
}
// Executable: unsigned.
const kUnsigned = { body: kScriptToExecute['body'] };
generate_test(kUnsigned, "", EXPECT_LOADED,
              "No signature, no integrity check: loads.");

generate_test(kUnsigned, "ed25519-???", EXPECT_LOADED,
              "No signature, malformed integrity check: loads.");

generate_test(kUnsigned, `ed25519-${kValidKeys['rfc']}`, EXPECT_BLOCKED,
              "No signature, valid integrity check: loads.");

// Executable and non-executable scripts signed with RFC's test key.
const kSignedShouldExecute = {
  body: kScriptToExecute['body'],
  digest: `sha-256=:${kScriptToExecute['hash']}:`,
  signatureInput: `signature=("identity-digest";sf);alg="ed25519";keyid="${kValidKeys['rfc']}";tag="sri"`,
  signature: `signature=:${kScriptToExecute['signatures']['rfc']}:`
};
const kSignedShouldBlock = {
  body: kScriptToBlock['body'],
  digest: `sha-256=:${kScriptToBlock['hash']}:`,
  signatureInput: `signature=("identity-digest";sf);alg="ed25519";keyid="${kValidKeys['rfc']}";tag="sri"`,
  signature: `signature=:${kScriptToBlock['signatures']['rfc']}:`
};

// Should load:
generate_test(kSignedShouldExecute, "", EXPECT_LOADED,
              "Valid signature, no integrity check: loads.");
generate_test(kSignedShouldExecute, "ed25519-???", EXPECT_LOADED,
              "Valid signature, malformed integrity check: loads.");
generate_test(kSignedShouldExecute, `ed25519-${kValidKeys['rfc']}`, EXPECT_LOADED,
              "Valid signature, valid integrity check: loads.");
generate_test(kSignedShouldExecute, `ed25519-${kValidKeys['rfc']} ed25519-${kValidKeys['arbitrary']}`, EXPECT_LOADED,
              "Valid signature, one matching integrity check: loads.");

// Should block:
generate_test(kSignedShouldBlock, `ed25519-${kValidKeys['arbitrary']}`, EXPECT_BLOCKED,
              "Valid signature, mismatched integrity check: blocked.");

// Executable and non-executable scripts signed with RFC's test key and the arbitrary key:
const kMultiplySignedShouldExecute = {
  body: kScriptToExecute['body'],
  digest: `sha-256=:${kScriptToExecute['hash']}:`,
  signatureInput: `signature1=("identity-digest";sf);alg="ed25519";keyid="${kValidKeys['rfc']}";tag="sri", ` +
                  `signature2=("identity-digest";sf);alg="ed25519";keyid="${kValidKeys['arbitrary']}";tag="sri"`,
  signature: `signature1=:${kScriptToExecute['signatures']['rfc']}:, ` +
             `signature2=:${kScriptToExecute['signatures']['arbitrary']}:`
};
const kMultiplySignedShouldBlock = {
  body: kScriptToBlock['body'],
  digest: `sha-256=:${kScriptToBlock['hash']}:`,
  signatureInput: `signature1=("identity-digest";sf);alg="ed25519";keyid="${kValidKeys['rfc']}";tag="sri", ` +
                  `signature2=("identity-digest";sf);alg="ed25519";keyid="${kValidKeys['arbitrary']}";tag="sri"`,
  signature: `signature1=:${kScriptToBlock['signatures']['rfc']}:, ` +
             `signature2=:${kScriptToBlock['signatures']['arbitrary']}:`
};
generate_test(kMultiplySignedShouldExecute, "", EXPECT_LOADED,
              "Valid signatures, no integrity check: loads.");
generate_test(kMultiplySignedShouldExecute, "ed25519-???", EXPECT_LOADED,
              "Valid signatures, malformed integrity check: loads.");
generate_test(kMultiplySignedShouldExecute, `ed25519-${kValidKeys['rfc']}`, EXPECT_LOADED,
              "Valid signatures, integrity check matches one: loads.");
generate_test(kMultiplySignedShouldExecute, `ed25519-${kValidKeys['arbitrary']}`, EXPECT_LOADED,
              "Valid signatures, integrity check matches the other: loads.");
generate_test(kMultiplySignedShouldExecute, `ed25519-${kValidKeys['rfc']} ed25519-${kValidKeys['arbitrary']}`, EXPECT_LOADED,
              "Valid signatures, integrity check matches both: loads.");

// Should block:
generate_test(kMultiplySignedShouldBlock, `ed25519-${kInvalidKey}`, EXPECT_BLOCKED,
              "Valid signatures, integrity check matches neither: blocked.");
