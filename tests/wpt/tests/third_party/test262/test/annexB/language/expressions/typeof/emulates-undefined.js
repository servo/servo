// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-IsHTMLDDA-internal-slot-typeof
description: >
  `typeof` operator returns "undefined" for [[IsHTMLDDA]] object.
info: |
  Changes to the typeof Operator

  The following table entry is inserted into Table 35 immediately
  preceeding the entry for "Object (implements [[Call]])":

  Type of val: Object (has an [[IsHTMLDDA]] internal slot)
  Result: "undefined"
features: [IsHTMLDDA]
---*/

var IsHTMLDDA = $262.IsHTMLDDA;

assert(typeof IsHTMLDDA === "undefined", '=== "undefined"');
assert.sameValue(typeof IsHTMLDDA, "undefined");

assert(typeof IsHTMLDDA !== "object", '!== "object"');
assert.sameValue(typeof IsHTMLDDA === "object", false, '!== "object"');

assert(typeof IsHTMLDDA !== "function", '!== "function"');
assert.sameValue(typeof IsHTMLDDA === "function", false, '!== "function"');
