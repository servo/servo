// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: >
  If thisArg is a primitive, mapfn is called with a wrapper this-value or the
  global, according to the usual rules of sloppy mode
info: |
  6. If _mapping_ is *true*, then
    a. Let _mappedValue_ be Call(_mapfn_, _thisArg_, « _nextValue_, 𝔽(_k_) »).

  OrdinaryCallBindThis, when _F_.[[ThisMode]] is ~global~, where _F_ is the
  function object:
  6. Else,
    a. If _thisArgument_ is *undefined* or *null*, then
      i. Let _globalEnv_ be _calleeRealm_.[[GlobalEnv]].
      ii. Assert: _globalEnv_ is a Global Environment Record.
      iii. Let _thisValue_ be _globalEnv_.[[GlobalThisValue]].
    b. Else,
      i. Let _thisValue_ be ! ToObject(_thisArgument_).
      ii. NOTE: ToObject produces wrapper objects using _calleeRealm_.
flags: [async, noStrict]
includes: [asyncHelpers.js]
features: [Array.fromAsync]
---*/

asyncTest(async () => {
  await Array.fromAsync([1, 2, 3], async function () {
    assert.sameValue(
      this,
      globalThis,
      "the global should be bound as the this-value of mapfn when thisArg is undefined"
    );
  }, undefined);

  await Array.fromAsync([1, 2, 3], async function () {
    assert.sameValue(
      this,
      globalThis,
      "the global should be bound as the this-value of mapfn when thisArg is null"
    );
  }, null);

  await Array.fromAsync([1, 2, 3], async function () {
    assert.notSameValue(this, "string", "string thisArg should not be bound as the this-value of mapfn");
    assert.sameValue(typeof this, "object", "a String wrapper object should be bound as the this-value of mapfn when thisArg is a string")
    assert.sameValue(this.valueOf(), "string", "String wrapper object should have the same primitive value as thisArg");
  }, "string");

  await Array.fromAsync([1, 2, 3], async function () {
    assert.notSameValue(this, 3.1416, "number thisArg should be not bound as the this-value of mapfn");
    assert.sameValue(typeof this, "object", "a Number wrapper object should be bound as the this-value of mapfn when thisArg is a number")
    assert.sameValue(this.valueOf(), 3.1416, "Number wrapper object should have the same primitive value as thisArg");
  }, 3.1416);

  await Array.fromAsync([1, 2, 3], async function () {
    assert.notSameValue(this, 42n, "bigint thisArg should not be bound as the this-value of mapfn");
    assert.sameValue(typeof this, "object", "a BigInt wrapper object should be bound as the this-value of mapfn when thisArg is a bigint")
    assert.sameValue(this.valueOf(), 42n, "BigInt wrapper object should have the same primitive value as thisArg");
  }, 42n);

  await Array.fromAsync([1, 2, 3], async function () {
    assert.notSameValue(this, true, "boolean thisArg should not be bound as the this-value of mapfn");
    assert.sameValue(typeof this, "object", "a Boolean wrapper object should be bound as the this-value of mapfn when thisArg is a boolean")
    assert.sameValue(this.valueOf(), true, "Boolean wrapper object should have the same primitive value as thisArg");
  }, true);

  const symbolThis = Symbol("symbol");
  await Array.fromAsync([1, 2, 3], async function () {
    assert.notSameValue(this, symbolThis, "symbol thisArg should not be bound as the this-value of mapfn");
    assert.sameValue(typeof this, "object", "a Symbol wrapper object should be bound as the this-value of mapfn when thisArg is a symbol")
    assert.sameValue(this.valueOf(), symbolThis, "Symbol wrapper object should have the same primitive value as thisArg");
  }, symbolThis);
});
