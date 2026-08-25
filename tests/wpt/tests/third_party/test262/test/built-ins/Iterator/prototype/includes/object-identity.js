// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.includes
description: >
  Includes compares objects by identity
features: [iterator-includes]
---*/

let o = {
  get toString() {
    throw new Test262Error();
  },
  get valueOf() {
    throw new Test262Error();
  }
};
let arr = [o];

assert.sameValue(arr.values().includes({
  get toString() {
    throw new Test262Error();
  },
  get valueOf() {
    throw new Test262Error();
  }
}), false);

assert.sameValue(arr.values().includes(""), false);

assert.sameValue(arr.values().includes(o), true);

assert.sameValue([].values().includes(o), false);
