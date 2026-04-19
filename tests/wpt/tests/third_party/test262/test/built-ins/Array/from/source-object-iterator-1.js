// Copyright 2015 Microsoft Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: Source object has iterator which throws
esid: sec-array.from
es6id: 22.1.2.1
features: [Symbol.iterator]
---*/

var array = [2, 4, 8, 16, 32, 64, 128];
var obj = {
  [Symbol.iterator]() {
    return {
      index: 0,
      next() {
        throw new Test262Error();
      },
      isDone: false,
      get val() {
        this.index++;
        if (this.index > 7) {
          this.isDone = true;
        }
        return 1 << this.index;
      }
    };
  }
};
assert.throws(Test262Error, function() {
  Array.from(obj);
}, 'Array.from(obj) throws a Test262Error exception');
