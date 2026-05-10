// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.every
description: >
  Arguments and this value are evaluated in the correct order
info: |
  %Iterator.prototype%.every ( predicate )

includes: [compareArray.js]
features: [iterator-helpers]
flags: []
---*/
let effects = [];

assert.throws(TypeError, function () {
  Iterator.prototype.every.call(
    {
      get next() {
        effects.push('get next');
        return function () {
          return { done: true, value: undefined };
        };
      },
    },
    null
  );
});

assert.compareArray(effects, []);
