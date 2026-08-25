// Copyright 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Properties of the groups object are created with CreateDataProperty
includes: [compareArray.js, propertyHelper.js]
esid: sec-regexpbuiltinexec
features: [regexp-named-groups]
info: |
  Runtime Semantics: RegExpBuiltinExec ( R, S )
    25. For each integer i such that i > 0 and i ≤ n
      f. If the ith capture of R was defined with a GroupName,
        i. Let s be the StringValue of the corresponding RegExpIdentifierName.
        ii. Perform ! CreateDataProperty(groups, s, capturedValue).
---*/

// Properties created on result.groups in textual order.
assert.compareArray(["fst", "snd"], Object.getOwnPropertyNames(
    /(?<fst>.)|(?<snd>.)/u.exec("abcd").groups));

// Properties are created with Define, not Set
let counter = 0;
Object.defineProperty(Object.prototype, 'x', {set() { counter++; }});
let match = /(?<x>.)/.exec('a');
let groups = match.groups;
assert.sameValue(counter, 0);

// Properties are writable, enumerable and configurable
// (from CreateDataProperty)
verifyProperty(groups, "x", {
  writable: true,
  enumerable: true,
  configurable: true,
});
