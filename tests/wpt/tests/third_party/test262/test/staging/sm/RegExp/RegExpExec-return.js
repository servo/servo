// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  RegExpExec should throw if returned value is not an object nor null.
info: bugzilla.mozilla.org/show_bug.cgi?id=887016
esid: pending
---*/

for (var ret of [null, {}, [], /a/]) {
  assert.sameValue(RegExp.prototype[Symbol.match].call({
    get global() {
      return false;
    },
    exec(S) {
      return ret;
    }
  }, "foo"), ret);
}

for (ret of [undefined, 1, true, false, Symbol.iterator]) {
  assert.throws(TypeError, () => {
    RegExp.prototype[Symbol.match].call({
      get global() {
        return false;
      },
      exec(S) {
        return ret;
      }
    }, "foo");
  });
}
