'use strict';

// See https://www.iana.org/assignments/cose/cose.xhtml#key-type
const cose_key_type_ec2 = 2;
const cose_key_type_rsa = 3;

// Decode |encoded| using a base64url decoding.
function base64urlToUint8Array(encoded) {
  return Uint8Array.from(base64urlDecode(encoded), c => c.charCodeAt(0));
}

// The result of a browser bound key verification.
const BrowserBoundKeyVerificationResult = Object.freeze({
  // No browser bound key was included.
  NoBrowserBoundKey: 'NoBrowserBoundKey',
  // A browser bound key was included and the cryptographic signature verifies.
  BrowserBoundKeySignatureVerified: 'BrowserBoundKeySignatureVerified',
});

// This function takes a credential and verifies either that no BBK was
// included (no browser bound public key, and no browser bound signature)
// or that the BBK was included (browser bound public key, browser bound
// signature, and the signature cryptographically verifies).
//
// Returns a BrowserBoundKeyVerificationResult informing the conditions of
// successful verification.
async function verifyBrowserBoundKey(credential, expectedKeyTypes) {
  const clientExtensionResults = credential.getClientExtensionResults();
  const signatureArray =
      clientExtensionResults?.payment?.browserBoundSignature?.signature;
  const clientData = JSON.parse(String.fromCharCode.apply(
      null, new Uint8Array(credential.response.clientDataJSON)));
  const publicKeyCoseKeyEncoded = clientData?.payment?.browserBoundPublicKey;
  assert_equals(
      signatureArray !== undefined, publicKeyCoseKeyEncoded !== undefined,
      'Either both or none of the browser bound public key and signature must ' +
          'be present, but only one was present.')
  if (signatureArray == undefined) {
    return BrowserBoundKeyVerificationResult.NoBrowserBoundKey;
  }
  assertBrowserBoundSignatureInClientExtensionResults(clientExtensionResults);
  await assertBrowserBoundKeySignature(
      credential.response.clientDataJSON, signatureArray, expectedKeyTypes);
  return BrowserBoundKeyVerificationResult.BrowserBoundKeySignatureVerified;
}

function getBrowserBoundPublicKeyFromCredential(credential) {
  const clientData = JSON.parse(String.fromCharCode.apply(
      null, new Uint8Array(credential.response.clientDataJSON)));
  return clientData?.payment?.browserBoundPublicKey;
}

function assertNoBrowserBoundPublicKeyInCredential(credential, message) {
  const clientData = JSON.parse(String.fromCharCode.apply(
      null, new Uint8Array(credential.response.clientDataJSON)));
  assert_equals(clientData?.payment?.browserBoundPublicKey, undefined, message);
}

function assertBrowserBoundSignatureInClientExtensionResults(
    clientExtensionResults) {
  assert_not_equals(
      clientExtensionResults.payment, undefined,
      'getClientExtensionResults().payment is not undefined');
  assert_not_equals(
      clientExtensionResults.payment.browserBoundSignature, undefined,
      'getClientExtensionResults().payment is not undefined');
  assert_not_equals(
      clientExtensionResults.payment.browserBoundSignature.signature, undefined,
      'getClientExtensionResults().payment.signature is not undefined');
}

async function assertBrowserBoundKeySignature(
    clientDataJSON, signatureArray, expectedKeyTypes) {
  const clientData = JSON.parse(
      String.fromCharCode.apply(null, new Uint8Array(clientDataJSON)));
  assert_not_equals(
      clientData.payment, undefined,
      `Deserialized clientData, ${
          JSON.stringify(clientData)}, should contain a 'payment' member`);
  assert_not_equals(
      clientData.payment.browserBoundPublicKey, undefined,
      `ClientData['payment'] should contain a 'browserBoundPublicKey' member.`);
  const browserBoundPublicKeyCoseKeyBase64 =
      clientData.payment.browserBoundPublicKey;
  const browserBoundPublicKeyCoseKeyEncoded =
      base64urlToUint8Array(browserBoundPublicKeyCoseKeyBase64);
  const keyType = getCoseKeyType(browserBoundPublicKeyCoseKeyEncoded);
  assert_true(
      expectedKeyTypes.includes(keyType),
      `KeyType, ${keyType}, was not one of the expected key types, ${
          expectedKeyTypes}`);
  if (keyType == cose_key_type_ec2) {
    // Verify the signature for a ES256 signature scheme.
    const browserBoundPublicKeyCoseKey =
        parseCosePublicKey(browserBoundPublicKeyCoseKeyEncoded);
    const jwkPublicKey = coseObjectToJWK(browserBoundPublicKeyCoseKey);
    const key = await crypto.subtle.importKey(
        'jwk', jwkPublicKey, {name: 'ECDSA', namedCurve: 'P-256'},
        /*extractable=*/ false, ['verify']);
    const signature =
        convertDERSignatureToSubtle(new Uint8Array(signatureArray));
    assert_true(await crypto.subtle.verify(
        {name: 'ECDSA', hash: 'SHA-256'}, key, signature, clientDataJSON));
  }
  // TODO: Verify the signature in case of an RS256 signature scheme.
}
