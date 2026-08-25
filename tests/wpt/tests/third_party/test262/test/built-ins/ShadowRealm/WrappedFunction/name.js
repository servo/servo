// Copyright (C) 2022 Chengzhong Wu. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-wrappedfunctioncreate
description: >
  The value of WrappedFunction.name is copied from the target function
info: |
  WrappedFunctionCreate ( callerRealm: a Realm Record, Target: a function object, )

  ...
  7. Let result be CopyNameAndLength(wrapped, Target).
  ...

  CopyNameAndLength ( F: a function object, Target: a function object, prefix: a String, optional argCount: a Number, )

  ...
  6. Let targetName be ? Get(Target, "name").
  7. If Type(targetName) is not String, set targetName to the empty String.
  8. Perform ! SetFunctionName(F, targetName, prefix).

  SetFunctionName ( F, name [ , prefix ] )

  ...
  6. Return ! DefinePropertyOrThrow(F, "name", PropertyDescriptor { [[Value]]: name, [[Writable]]: false, [[Enumerable]]: false, [[Configurable]]: true }).

includes: [propertyHelper.js]
features: [ShadowRealm]
---*/

assert.sameValue(
  typeof ShadowRealm.prototype.evaluate,
  'function',
  'This test must fail if ShadowRealm.prototype.evaluate is not a function'
);

const r = new ShadowRealm();

let wrapped = r.evaluate(`
function fn() {}
fn;
`);
verifyProperty(wrapped, "name", {
  value: "fn",
  enumerable: false,
  writable: false,
  configurable: true,
});

// The name property is an accessor.
wrapped = r.evaluate(`
function fn() {}
Object.defineProperty(fn, 'name', {
  get: () => "bar",
  enumerable: false,
  configurable: true,
});
fn;
`);
verifyProperty(wrapped, "name", {
  value: "bar",
  enumerable: false,
  writable: false,
  configurable: true,
});

// The value of fn.name is not a string.
for (const name of [null, undefined, 0, '1n', false, NaN, Infinity, 'Symbol()', '[]', '{}']) {
  wrapped = r.evaluate(`
function fn() {}
Object.defineProperty(fn, 'name', {
  value: ${String(name)},
  enumerable: false,
  configurable: true,
});
fn;
`);
  verifyProperty(wrapped, "name", {
    value: "",
    enumerable: false,
    writable: false,
    configurable: true,
  });
}
