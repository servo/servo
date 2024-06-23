// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/resources/utils.js
// META: script=helpers.js

// The string "test" as ASCII bytes and base64url-encoded.
const test_bytes = new Uint8Array([0x74, 0x65, 0x73, 0x74]);
const test_b64 = "dGVzdA";

test(() => {
  let actual = PublicKeyCredential.parseRequestOptionsFromJSON({
    challenge: test_b64,
    timeout: 60000,
    rpId: "example.com",
    allowCredentials: [
      { type: "public-key", id: test_b64 },
    ],
    userVerification: "required",
    hints: ["hybrid", "security-key"],
  });
  let expected = {
    challenge: test_bytes,
    timeout: 60000,
    rpId: "example.com",
    allowCredentials: [
      { type: "public-key", id: test_bytes },
    ],
    userVerification: "required",
    hints: ["hybrid", "security-key"],
  };

  assert_equals(actual.rpId, expected.rpId);
  assert_true(bytesEqual(actual.challenge, expected.challenge));
  assert_equals(actual.timeout, expected.timeout);
  assert_equals(actual.allowCredentials.length, expected.allowCredentials.length);
  assert_equals(actual.allowCredentials[0].type, expected.allowCredentials[0].type);
  assert_true(bytesEqual(actual.allowCredentials[0].id, expected.allowCredentials[0].id));
  assert_equals(actual.userVerification, expected.userVerification);
  assertJsonEquals(actual.hints, expected.hints);
}, "parseRequestOptionsFromJSON()");

test(() => {
  let actual = PublicKeyCredential.parseRequestOptionsFromJSON({
    challenge: test_b64,
    extensions: {
      appid: "app id",
      largeBlob: {
        read: true,
      },
      getCredBlob: true,
      supplementalPubKeys: {
        scopes: ["spk scope"],
        attestation: "directest",
        attestationFormats: ["asn2"],
      },
      prf: {
        eval: {
          first: test_b64,
          second: test_b64,
        },
        evalByCredential: {
          "test cred": {
            first: test_b64,
            second: test_b64,
          },
        },
      },
    },
  });
  let expected = {
    challenge: test_b64,
    extensions: {
      appid: "app id",
      largeBlob: {
        read: true,
      },
      getCredBlob: true,
      supplementalPubKeys: {
        scopes: ["spk scope"],
        attestation: "directest",
        attestationFormats: ["asn2"],
      },
      prf: {
        eval: {
          first: test_bytes,
          second: test_bytes,
        },
        evalByCredential: {
          "test cred": {
            first: test_bytes,
            second: test_bytes,
          },
        },
      },
    },
  };

  assert_equals(actual.extensions.appid, expected.extensions.appid);
  assert_equals(actual.extensions.largeBlob.read, expected.extensions.largeBlob.read);
  assert_equals(actual.extensions.getCredBlob, expected.extensions.getCredBlob);
  assertJsonEquals(actual.extensions.supplementalPubKeys, expected.extensions.supplementalPubKeys);
  let prfValuesEquals = (a, b) => {
    return bytesEqual(a.first, b.first) && bytesEqual(a.second, b.second);
  };
  assert_true(prfValuesEquals(actual.extensions.prf.eval, expected.extensions.prf.eval), "prf eval");
  assert_true(prfValuesEquals(actual.extensions.prf.evalByCredential["test cred"], expected.extensions.prf.evalByCredential["test cred"]), "prf ebc");
}, "parseRequestOptionsFromJSON() with extensions");
