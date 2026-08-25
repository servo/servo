// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Leaving for-in and try should handle stack value in correct order
info: bugzilla.mozilla.org/show_bug.cgi?id=1332881
esid: pending
---*/

var called = 0;
function reset() {
  called = 0;
}
var obj = {
  [Symbol.iterator]() {
    return {
      next() {
        return { value: 10, done: false };
      },
      return() {
        called++;
        return {};
      }
    };
  }
};

var a = (function () {
    for (var x in [0]) {
        try {} finally {
            return 11;
        }
    }
})();
assert.sameValue(a, 11);

reset();
var b = (function () {
    for (var x of obj) {
        try {} finally {
            return 12;
        }
    }
})();
assert.sameValue(called, 1);
assert.sameValue(b, 12);

reset();
var c = (function () {
    for (var x in [0]) {
        for (var y of obj) {
            try {} finally {
                return 13;
            }
        }
    }
})();
assert.sameValue(called, 1);
assert.sameValue(c, 13);

reset();
var d = (function () {
    for (var x in [0]) {
        for (var y of obj) {
            try {} finally {
                for (var z in [0]) {
                    for (var w of obj) {
                        try {} finally {
                            return 14;
                        }
                    }
                }
            }
        }
    }
})();
assert.sameValue(called, 2);
assert.sameValue(d, 14);
