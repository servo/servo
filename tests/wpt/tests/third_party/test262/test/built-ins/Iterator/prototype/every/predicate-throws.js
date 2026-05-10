// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.every
description: >
  Closes iterator and throws when predicate throws
info: |
  %Iterator.prototype%.every ( predicate )

  4.d. Let result be Completion(Call(predicate, undefined, « value, 𝔽(counter) »)).
  4.e. IfAbruptCloseIterator(result, iterated).

features: [iterator-helpers]
flags: []
---*/
let returnCalls = 0;

class TestIterator extends Iterator {
  next() {
    return {
      done: false,
      value: 1,
    };
  }
  return() {
    ++returnCalls;
    return {};
  }
}

let iterator = new TestIterator();

let callbackCalls = 0;

assert.throws(Test262Error, function () {
  iterator.every(() => {
    ++callbackCalls;
    throw new Test262Error();
  });
});

assert.sameValue(callbackCalls, 1);
assert.sameValue(returnCalls, 1);
