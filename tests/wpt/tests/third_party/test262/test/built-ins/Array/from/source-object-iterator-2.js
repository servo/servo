// Copyright 2015 Microsoft Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: Source object has iterator
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
        return {
          value: this.val,
          done: this.isDone
        };
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
var a = Array.from.call(Object, obj);
assert.sameValue(typeof a, typeof {}, 'The value of `typeof a` is expected to be typeof {}');
for (var j = 0; j < a.length; j++) {
  assert.sameValue(a[j], array[j], 'The value of a[j] is expected to equal the value of array[j]');
}
