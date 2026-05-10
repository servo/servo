// Copyright (C) 2021 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-shadowrealm.prototype.importvalue
description: >
  ShadowRealm.prototype.importValue coerces specifier to string.
features: [ShadowRealm]
---*/

assert.sameValue(
  typeof ShadowRealm.prototype.importValue,
  'function',
  'This test must fail if ShadowRealm.prototype.importValue is not a function'
);

const r = new ShadowRealm();
let count = 0;

const specifier = Object.create(null);

// A - valueOF

specifier.valueOf = function() {
  count += 1;
  throw new Test262Error();
};

assert.throws(Test262Error, () => {
  r.importValue(specifier);
}, 'ToString(specifier) returns abrupt from valueOf');

assert.sameValue(count, 1, 'ToString calls the valueOf method');


// B - toString

count = 0;

specifier.valueOf = function() {
  count += 1000;
  throw new Error('valueOf is not reached if toString is present');
};

specifier.toString = function() {
  count += 1;
  throw new Test262Error();
};

assert.throws(Test262Error, () => {
  r.importValue(specifier);
}, 'ToString(specifier) returns abrupt from toString');

assert.sameValue(count, 1, 'ToString calls the toString method');

// C - @@toPrimitive

count = 0;

specifier[Symbol.toPrimitive] = function() {
  count += 1;
  throw new Test262Error();
};

specifier.toString = function() {
  count += 1000;
  throw new Error('toString is not reached if @@toPrimitive is present');
};

assert.throws(Test262Error, () => {
  r.importValue(specifier);
}, 'ToString(specifier) returns abrupt from @@toPrimitive');

assert.sameValue(count, 1, 'ToString calls the @@toPrimitive method');
