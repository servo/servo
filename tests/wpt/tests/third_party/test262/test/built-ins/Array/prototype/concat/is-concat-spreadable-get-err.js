// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.concat
description: Error thrown when accessing `Symbol.isConcatSpreadable` property
info: |
    1. Let O be ToObject(this value).
    2. ReturnIfAbrupt(O).
    3. Let A be ArraySpeciesCreate(O, 0).
    4. ReturnIfAbrupt(A).
    5. Let n be 0.
    6. Let items be a List whose first element is O and whose subsequent
       elements are, in left to right order, the arguments that were passed to
       this function invocation.
    7. Repeat, while items is not empty
      a. Remove the first element from items and let E be the value of the element.
      b. Let spreadable be IsConcatSpreadable(E).
      c. ReturnIfAbrupt(spreadable).

    ES6 22.1.3.1.1: Runtime Semantics: IsConcatSpreadable ( O )

    1. If Type(O) is not Object, return false.
    2. Let spreadable be Get(O, @@isConcatSpreadable).
    3. ReturnIfAbrupt(spreadable).
features: [Symbol.isConcatSpreadable]
---*/

var o = {};

Object.defineProperty(o, Symbol.isConcatSpreadable, {
  get: function() {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  Array.prototype.concat.call(o);
}, 'Array.prototype.concat.call(o) throws a Test262Error exception');
