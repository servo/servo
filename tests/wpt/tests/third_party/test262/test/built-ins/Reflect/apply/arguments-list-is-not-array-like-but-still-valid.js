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
  Let len be ? LengthOfArrayLike(obj).
  Let list be a new empty List.
  Let index be 0.
  Repeat, while index < len,
    Let indexName be ! ToString(index).
    Let next be ? Get(obj, indexName).
    If Type(next) is not an element of elementTypes, throw a TypeError exception.
    Append next as the last element of list.
    Set index to index + 1.
  Return list.
includes: [compareArray.js]
features: [Reflect, arrow-function, Symbol]
---*/

let count = 0;

function fn(...args) {
  count++;
  return args;
}

let f_unction = new Function();

Object.defineProperty(f_unction, "length", {
  get() {
    return 1;
  }
});

assert.compareArray(Reflect.apply(fn, null, f_unction), [undefined]);

let object = new Object();

Object.defineProperty(object, "length", {
  get() {
    return 1;
  }
});

assert.compareArray(Reflect.apply(fn, null, object), [undefined]);

let number = new Number();

Object.defineProperty(number, "length", {
  get() {
    return 1;
  }
});

assert.compareArray(Reflect.apply(fn, null, number), [undefined]);

let boolean = new Boolean();

Object.defineProperty(boolean, "length", {
  get() {
    return 1;
  }
});

assert.compareArray(Reflect.apply(fn, null, boolean), [undefined]);

assert.sameValue(count, 4, 'The value of `count` is 1');
