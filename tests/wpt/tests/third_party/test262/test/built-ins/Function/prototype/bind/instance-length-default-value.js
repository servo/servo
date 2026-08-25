// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-function.prototype.bind
description: >
  "length" value of a bound function defaults to 0.
  Non-own and non-number "length" values of target function are ignored.
info: |
  Function.prototype.bind ( thisArg, ...args )

  [...]
  5. Let targetHasLength be ? HasOwnProperty(Target, "length").
  6. If targetHasLength is true, then
    a. Let targetLen be ? Get(Target, "length").
    b. If Type(targetLen) is not Number, let L be 0.
    c. Else,
      i. Set targetLen to ! ToInteger(targetLen).
      ii. Let L be the larger of 0 and the result of targetLen minus the number of elements of args.
  7. Else, let L be 0.
  8. Perform ! SetFunctionLength(F, L).
  [...]
features: [Symbol]
---*/

function foo() {}

Object.defineProperty(foo, "length", {value: undefined});
assert.sameValue(foo.bind(null, 1).length, 0, "undefined");

Object.defineProperty(foo, "length", {value: null});
assert.sameValue(foo.bind(null, 1).length, 0, "null");

Object.defineProperty(foo, "length", {value: true});
assert.sameValue(foo.bind(null, 1).length, 0, "boolean");

Object.defineProperty(foo, "length", {value: "1"});
assert.sameValue(foo.bind(null, 1).length, 0, "string");

Object.defineProperty(foo, "length", {value: Symbol("1")});
assert.sameValue(foo.bind(null, 1).length, 0, "symbol");

Object.defineProperty(foo, "length", {value: new Number(1)});
assert.sameValue(foo.bind(null, 1).length, 0, "Number object");


function bar() {}
Object.setPrototypeOf(bar, {length: 42});
assert(delete bar.length);

var bound = Function.prototype.bind.call(bar, null, 1);
assert.sameValue(bound.length, 0, "not own");
