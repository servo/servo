// META: script=/webauthn/helpers.js

// RP ID remote validation URL blocked by Connection Allowlist.
promise_test(async (t) => {
  const createArgs = cloneObject(createCredentialDefaultArgs);
  createArgs.options.publicKey.rp.id = '{{hosts[][www]}}';
  let challengeBytes = new Uint8Array(16);
  window.crypto.getRandomValues(challengeBytes);
  createArgs.options.publicKey.challenge = challengeBytes;
  createArgs.options.publicKey.user.id = new Uint8Array(16);

  await promise_rejects_dom(
      t, 'NotAllowedError', navigator.credentials.create(createArgs.options),
      'Blocked by Connection Allowlist');
}, 'WebAuth remote validation blocked by Connection Allowlist');

// RP ID remote validation URL allowed by Connection Allowlist. It does not
// fail with "NotAllowedError" but "SecurityError" instead, due to port mismatch
// in test env.
//
// The remote validation URL must have the default port 443 according to WebAuth
// spec: https://w3c.github.io/webauthn/#sctn-validating-relation-origin.
// It is difficult to make web platform test server to listen to port 443 so
// that `navigator.credentials.create` succeeds. Instead, we rely on observing
// a different error than the one shown when the request is blocked to verify
// Connection Allowlist is correctly enforced.
promise_test(async (t) => {
  const createArgs = cloneObject(createCredentialDefaultArgs);
  createArgs.options.publicKey.rp.id = '{{hosts[alt][]}}';
  let challengeBytes = new Uint8Array(16);
  window.crypto.getRandomValues(challengeBytes);
  createArgs.options.publicKey.challenge = challengeBytes;
  createArgs.options.publicKey.user.id = new Uint8Array(16);

  // Should fail with SecurityError (fetch failed), instead of NotAllowedError
  // (blocked by Connection Allowlist).
  await promise_rejects_dom(
      t, 'SecurityError', navigator.credentials.create(createArgs.options),
      'Allowed by Connection Allowlist but fetch fails (port mismatch)');
}, 'WebAuth remote validation allowed by Connection Allowlist');
