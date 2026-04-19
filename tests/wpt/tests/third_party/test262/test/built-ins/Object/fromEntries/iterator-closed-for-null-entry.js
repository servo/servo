// Copyright (C) 2018 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.fromentries
description: Closes iterators when they return entries which are null.
info: |
  Object.fromEntries ( iterable )

  ...
  4. Let stepsDefine be the algorithm steps defined in CreateDataPropertyOnObject Functions.
  5. Let adder be CreateBuiltinFunction(stepsDefine, « »).
  6. Return ? AddEntriesFromIterable(obj, iterable, adder).

  AddEntriesFromIterable ( target, iterable, adder )

  ...
  4. Repeat,
    ...
    d. If Type(nextItem) is not Object, then
      i. Let error be ThrowCompletion(a newly created TypeError object).
      ii. Return ? IteratorClose(iteratorRecord, error).

features: [Symbol.iterator, Object.fromEntries]
---*/

var returned = false;
var iterable = {
  [Symbol.iterator]: function() {
    var advanced = false;
    return {
      next: function() {
        if (advanced) {
          throw new Test262Error('should only advance once');
        }
        advanced = true;
        return {
          done: false,
          value: null,
        };
      },
      return: function() {
        if (returned) {
          throw new Test262Error('should only return once');
        }
        returned = true;
      },
    };
  },
};

assert.sameValue(typeof Object.fromEntries, 'function');
assert.throws(TypeError, function() {
  Object.fromEntries(iterable);
});

assert(returned, 'iterator should be closed when entry is null');
