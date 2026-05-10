// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-aggregate-error
description: >
  Iteration of errors
info: |
  AggregateError ( errors, message )

  ...
  3. Let errorsList be ? IterableToList(errors).
  4. Set O.[[AggregateErrors]] to errorsList.
  ...
  6. Return O.

  Runtime Semantics: IterableToList ( items [ , method ] )

  1. If method is present, then
    ...
  2. Else,
    b. Let iteratorRecord be ? GetIterator(items, sync).
  3. Let values be a new empty List.
  4. Let next be true.
  5. Repeat, while next is not false
    a. Set next to ? IteratorStep(iteratorRecord).
    b. If next is not false, then
      i. Let nextValue be ? IteratorValue(next).
      ii. Append nextValue to the end of the List values.
  6. Return values.

  GetIterator ( obj [ , hint [ , method ] ] )

  ...
  3. If method is not present, then
    a. If hint is async, then
      ...
    b. Otherwise, set method to ? GetMethod(obj, @@iterator).
  4. Let iterator be ? Call(method, obj).
  5. If Type(iterator) is not Object, throw a TypeError exception.
  6. Let nextMethod be ? GetV(iterator, "next").
  ...
  8. Return iteratorRecord.
features: [AggregateError, Symbol.iterator]
includes: [compareArray.js]
---*/

var count = 0;
var values = [];
var case1 = {
  [Symbol.iterator]() {
    return {
      next() {
        count += 1;
        return {
          done: count === 3,
          get value() {
            values.push(count)
          }
        };
      }
    };
  }
};

new AggregateError(case1);

assert.sameValue(count, 3);
assert.compareArray(values, [1, 2]);

assert.throws(TypeError, () => {
  new AggregateError();
}, 'GetMethod(obj, @@iterator) returns undefined');
