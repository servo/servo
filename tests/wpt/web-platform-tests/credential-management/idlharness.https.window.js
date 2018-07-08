// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/webappsec-credential-management/

'use strict';

promise_test(async () => {
  const idl = await fetch('/interfaces/credential-management.idl').then(r => r.text());
  const html = await fetch('/interfaces/html.idl').then(r => r.text());

  var idl_array = new IdlArray();
  idl_array.add_idls(idl);
  idl_array.add_dependency_idls(html);
  idl_array.add_objects({
    CredentialsContainer: ['navigator.credentials'],
    PasswordCredential: [
      `new PasswordCredential({
        id: "id",
        password: "pencil",
        iconURL: "https://example.com/",
        name: "name"
      })`
    ],
    FederatedCredential: [
      `new FederatedCredential({
        id: "id",
        provider: "https://example.com",
        iconURL: "https://example.com/",
        name: "name"
      })`
    ]
  });
  idl_array.test();
})
