// Copyright (C) 2018 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.fromentries
description: Closes iterators when accessing an entry's value throws.
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
    g. Let v be Get(nextItem, "1").
    h. If v is an abrupt completion, return ? IteratorClose(iteratorRecord, v).

features: [Symbol.iterator, Object.fromEntries]
---*/

function DummyError() {}

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
          value: {
            get '0'() {
              return 'key';
            },
            get '1'() {
              throw new DummyError();
            },
          },
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

assert.throws(DummyError, function() {
  Object.fromEntries(iterable);
});

assert(returned, 'iterator should be closed when entry value property access throws');
