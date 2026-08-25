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

  1. If argCount is undefined, then set argCount to 0.
  2. Let L be 0.
  3. Let targetHasLength be ? HasOwnProperty(Target, "length").
  4. If targetHasLength is true, then
    a. Let targetLen be ? Get(Target, "length").
    b. If Type(targetLen) is Number, then
        i. If targetLen is +∞𝔽, set L to +∞.
        ii. Else if targetLen is -∞𝔽, set L to 0.
        iii. Else,
            1. Let targetLenAsInt be ! ToIntegerOrInfinity(targetLen).
            2. Assert: targetLenAsInt is finite.
            3. Set L to max(targetLenAsInt - argCount, 0).
  5. Perform ! SetFunctionLength(F, L).
  ...

  SetFunctionLength ( F, length )

  ...
  2. Return ! DefinePropertyOrThrow(F, "length", PropertyDescriptor { [[Value]]: 𝔽(length), [[Writable]]: false, [[Enumerable]]: false, [[Configurable]]: true }).

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
function fn(foo, bar) {}
fn;
`);
verifyProperty(wrapped, "length", {
  value: 2,
  enumerable: false,
  writable: false,
  configurable: true,
});


wrapped = r.evaluate(`
function fn() {}
delete fn.length;
fn;
`);
verifyProperty(wrapped, "length", {
  value: 0,
  enumerable: false,
  writable: false,
  configurable: true,
});


wrapped = r.evaluate(`
function fn() {}
Object.defineProperty(fn, 'length', {
  get: () => Infinity,
  enumerable: false,
  configurable: true,
});
fn;
`);
verifyProperty(wrapped, "length", {
  value: Infinity,
  enumerable: false,
  writable: false,
  configurable: true,
});


wrapped = r.evaluate(`
function fn() {}
Object.defineProperty(fn, 'length', {
  get: () => -Infinity,
  enumerable: false,
  configurable: true,
});
fn;
`);
verifyProperty(wrapped, "length", {
  value: 0,
  enumerable: false,
  writable: false,
  configurable: true,
});


wrapped = r.evaluate(`
function fn() {}
Object.defineProperty(fn, 'length', {
  get: () => -1,
  enumerable: false,
  configurable: true,
});
fn;
`);
verifyProperty(wrapped, "length", {
  value: 0,
  enumerable: false,
  writable: false,
  configurable: true,
});
