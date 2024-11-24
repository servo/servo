// META: script=/resources/testharness-shadowrealm-outer.js
// META: script=/resources/idlharness-shadowrealm.js
idl_test_shadowrealm(
  ["webidl"],
  [],
  idl_array => {
    idl_array.add_objects({
      DOMException: ["new DOMException()",
                     'new DOMException("my message")',
                     'new DOMException("my message", "myName")']
    });
  }
);
