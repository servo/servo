// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

idl_test(
    ['trusted-types.tentative'],
    ['dom', 'html'],
    idl_array => {
      idl_array.add_objects({
        TrustedTypePolicyFactory: ['window.TrustedTypes'],
        TrustedTypePolicy: ['window.TrustedTypes.createPolicy("SomeName", { createHTML: s => s })'],
        TrustedHTML: ['window.TrustedTypes.createPolicy("SomeName1", { createHTML: s => s }).createHTML("A string")'],
        TrustedScript: ['window.TrustedTypes.createPolicy("SomeName2", { createScript: s => s }).createScript("A string")'],
        TrustedScriptURL: ['window.TrustedTypes.createPolicy("SomeName3", { createScriptURL: s => s }).createScriptURL("A string")'],
        TrustedURL: ['window.TrustedTypes.createPolicy("SomeName4", { createURL: s => s }).createURL("A string")']
      });
    },
    'Trusted Types'
);
