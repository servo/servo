// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/resources/utils.js
// META: script=helpers.js

// The string "test" as ASCII bytes and base64url-encoded.
const test_bytes = new Uint8Array([0x74, 0x65, 0x73, 0x74]);
const test_b64 = "dGVzdA";

test(() => {
  let actual = PublicKeyCredential.parseCreationOptionsFromJSON({
    rp: {
      id: "example.com",
      name: "Example Inc",
    },
    user: {
      id: test_b64,
      name: "test@example.com",
      displayName: "test user"
    },
    challenge: test_b64,
    pubKeyCredParams: [
      {
        type: "public-key",
        alg: -7,
      },
    ],
  });
  let expected = {
    rp: {
      id: "example.com",
      name: "Example Inc",
    },
    user: {
      id: test_bytes,
      name: "test@example.com",
      displayName: "test user"
    },
    challenge: test_bytes,
    pubKeyCredParams: [
      {
        type: "public-key",
        alg: -7,
      },
    ],
    // The spec defaults the following fields:
    attestation: "none",
    hints: [],
  };

  assertJsonEquals(actual.rp, expected.rp);
  assert_true(userEntityEquals(actual.user, expected.user));
  assert_true(bytesEqual(actual.challenge, expected.challenge));
  assertJsonEquals(actual.pubKeyCredParams, expected.pubKeyCredParams, "pk");
  assert_equals(actual.attestation, expected.attestation);
  assertJsonEquals(actual.hints, expected.hints);
}, "parseCreationOptionsFromJSON()");

test(() => {
  assert_throws_dom("EncodingError", () => {
    PublicKeyCredential.parseCreationOptionsFromJSON({
      rp: {
        id: "example.com",
        name: "Example Inc",
      },
      user: {
        id: "not valid base64url",
        name: "test@example.com",
        displayName: "test user"
      },
      challenge: "not valid base64url",
      pubKeyCredParams: [
        {
          type: "public-key",
          alg: -7,
        },
      ],
    });
  });
}, "parseCreationOptionsFromJSON() throws EncodingError");

test(() => {
  let actual = PublicKeyCredential.parseCreationOptionsFromJSON({
    rp: {
      id: "example.com",
      name: "Example Inc",
    },
    user: {
      id: test_b64,
      name: "test@example.com",
      displayName: "test user"
    },
    challenge: test_b64,
    pubKeyCredParams: [
      {
        type: "public-key",
        alg: -7,
      },
    ],
    extensions: {
      appid: "app id",
      appidExclude: "app id exclude",
      hamcCreateSecret: true,
      uvm: true,
      credentialProtectionPolicy: "cred protect",
      enforceCredentialProtectionPolicy: true,
      minPinLength: true,
      credProps: true,
      largeBlob: {
        support: "large blob support",
        read: true,
        write: test_b64,
      },
      credBlob: test_b64,
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
    rp: {
      id: "example.com",
      name: "Example Inc",
    },
    user: {
      id: test_bytes,
      name: "test@example.com",
      displayName: "test user"
    },
    challenge: test_bytes,
    pubKeyCredParams: [
      {
        type: "public-key",
        alg: -7,
      },
    ],
    extensions: {
      appid: "app id",
      appidExclude: "app id exclude",
      hamcCreateSecret: true,
      uvm: true,
      credentialProtectionPolicy: "cred protect",
      enforceCredentialProtectionPolicy: true,
      minPinLength: true,
      credProps: true,
      largeBlob: {
        support: "large blob support",
        read: true,
        write: test_bytes,
      },
      credBlob: test_bytes,
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
      // The spec defaults the following fields:
      attestation: "none",
      hints: [],
    },
  };

  assert_equals(actual.extensions.appid, expected.extensions.appid);
  assert_equals(actual.extensions.appidExclude, expected.extensions.appidExclude);
  assert_equals(actual.extensions.hmacCreateSecret, expected.extensions.hmacCreateSecret);
  assert_equals(actual.extensions.uvm, expected.extensions.uvm);
  assert_equals(actual.extensions.credentialProtectionPolicy, expected.extensions.credentialProtectionPolicy);
  assert_equals(actual.extensions.enforceCredentialProtectionPolicy, expected.extensions.enforceCredentialProtectionPolicy);
  assert_equals(actual.extensions.minPinLength, expected.extensions.minPinLength);
  assert_equals(actual.extensions.credProps, expected.extensions.credProps);
  assert_equals(actual.extensions.largeBlob.support, expected.extensions.largeBlob.support, "X");
  assert_equals(actual.extensions.largeBlob.read, expected.extensions.largeBlob.read);

  assert_true(bytesEqual(actual.extensions.largeBlob.write, expected.extensions.largeBlob.write), "XX");

  assert_true(bytesEqual(actual.extensions.credBlob, expected.extensions.credBlob), "XXX");

  assert_equals(actual.extensions.getCredBlob, expected.extensions.getCredBlob);
  assertJsonEquals(actual.extensions.supplementalPubKeys, expected.extensions.supplementalPubKeys);
  let prfValuesEquals = (a, b) => {
    return bytesEqual(a.first, b.first) && bytesEqual(a.second, b.second);
  };
  assert_true(prfValuesEquals(actual.extensions.prf.eval, expected.extensions.prf.eval), "prf eval");
  assert_true(prfValuesEquals(actual.extensions.prf.evalByCredential["test cred"], expected.extensions.prf.evalByCredential["test cred"]), "prf ebc");
}, "parseCreationOptionsFromJSON() with extensions");
