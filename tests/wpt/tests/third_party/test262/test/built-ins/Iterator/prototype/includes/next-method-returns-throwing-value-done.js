// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.includes
description: >
  Underlying iterator next returns object with throwing value getter, but is already done
features: [iterator-includes]
---*/

let iterator = {
  __proto__: Iterator.prototype,
  next() {
    return {
      done: true,
      get value() {
        throw new Test262Error();
      },
    };
  }
};

assert.sameValue(iterator.includes(0), false);
