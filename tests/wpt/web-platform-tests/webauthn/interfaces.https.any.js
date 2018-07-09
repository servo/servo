// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/webauthn/

'use strict';

promise_test(async () => {
  const webauthnIdl = await fetch('/interfaces/webauthn.idl').then(r => r.text());
  const creds = await fetch('/interfaces/credential-management.idl').then(r => r.text());

  const idlArray = new IdlArray();
  idlArray.add_idls(webauthnIdl);
  idlArray.add_dependency_idls(creds);
  idlArray.add_objects({
    WebAuthentication: ["navigator.authentication"]
  });
  idlArray.test();
  done();
}, 'WebAuthn interfaces.');
