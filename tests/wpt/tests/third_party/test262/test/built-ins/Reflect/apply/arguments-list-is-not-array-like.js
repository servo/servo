// Copyright (C) 2015 the V8 project authors. All rights reserved.
// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-reflect.apply
description: >
  Return abrupt if argumentsList is not an ArrayLike object.
info: |
  Reflect.apply ( target, thisArgument, argumentsList )

  ...
  Let args be ? CreateListFromArrayLike(argumentsList).


  CreateListFromArrayLike (obj [, elementTypes] )

  ...
  If Type(obj) is not Object, throw a TypeError exception.
features: [Reflect, arrow-function, Symbol]
---*/

let count = 0;

function fn() {
  count++;
}

assert.throws(Test262Error, () => {
  Reflect.apply(fn, null, {
    get length() {
      throw new Test262Error();
    }
  });
}, '`Reflect.apply(fn, null, {get length() {throw new Test262Error();}})` throws a Test262Error exception');

assert.throws(TypeError, () => {
  Reflect.apply(fn, null /* empty */);
}, '`Reflect.apply(fn, null /* empty */)` throws a TypeError exception');

assert.throws(TypeError, () => {
  Reflect.apply(fn, null, Symbol());
}, '`Reflect.apply(fn, null, Symbol())` throws a TypeError exception');

assert.throws(TypeError, () => {
  Reflect.apply(fn, null, 1);
}, '`Reflect.apply(fn, null, 1)` throws a TypeError exception');

assert.throws(TypeError, () => {
  Reflect.apply(fn, null, Infinity);
}, '`Reflect.apply(fn, null, Infinity)` throws a TypeError exception');

assert.throws(TypeError, () => {
  Reflect.apply(fn, null, null);
}, '`Reflect.apply(fn, null, null)` throws a TypeError exception');

assert.throws(TypeError, () => {
  Reflect.apply(fn, null, undefined);
}, '`Reflect.apply(fn, null, undefined)` throws a TypeError exception');

assert.throws(TypeError, () => {
  Reflect.apply(fn, null, false);
}, '`Reflect.apply(fn, null, false)` throws a TypeError exception');

assert.throws(TypeError, () => {
  Reflect.apply(fn, null, true);
}, '`Reflect.apply(fn, null, true)` throws a TypeError exception');

assert.throws(TypeError, () => {
  Reflect.apply(fn, null, NaN);
}, '`Reflect.apply(fn, null, NaN)` throws a TypeError exception');


assert.sameValue(count, 0, 'The value of `count` is 0');
