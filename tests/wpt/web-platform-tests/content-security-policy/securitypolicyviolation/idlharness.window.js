// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/webappsec-csp/

'use strict';

idl_test(
  ['CSP'],
  ['dom', 'reporting'],
  idl_array => {
    idl_array.add_objects({
      SecurityPolicyViolationEvent: [
        'new SecurityPolicyViolationEvent("securitypolicyviolation")'
      ]
    })
  }
);
