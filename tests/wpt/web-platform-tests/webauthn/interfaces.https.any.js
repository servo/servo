// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

if (self.importScripts) {
  importScripts('/resources/testharness.js');
  importScripts('/resources/WebIDLParser.js', '/resources/idlharness.js');
}

// https://w3c.github.io/webauthn/

promise_test(async () => {
  const webauthnIdl = await fetch('/interfaces/webauthn.idl').then(r => r.text());

  const idlArray = new IdlArray();
  idlArray.add_idls(webauthnIdl);

  // static IDL tests
  idlArray.add_untested_idls('interface CredentialCreationOptions {};');
  idlArray.add_untested_idls('interface CredentialRequestOptions {};');
  idlArray.add_untested_idls("interface Navigator { };");
  idlArray.add_untested_idls("interface Credential { };");
  // TODO: change to "tested" for real browsers?
  idlArray.add_untested_idls("partial interface Navigator { readonly attribute WebAuthentication authentication; };");
  idlArray.add_objects({
    WebAuthentication: ["navigator.authentication"]
  });
  idlArray.test();
  done();
}, 'WebAuthn interfaces.');
