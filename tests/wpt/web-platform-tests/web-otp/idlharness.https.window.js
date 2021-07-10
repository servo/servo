// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/webauthn/

'use strict';

idl_test(
  ['web-otp'],
  ['credential-management'],
  idlArray => {
    idlArray.add_objects({
      // TODO: create an OTPCredential
    });
  }
);
