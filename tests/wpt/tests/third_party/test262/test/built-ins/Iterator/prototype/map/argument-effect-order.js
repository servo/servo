// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.map
description: >
  Arguments and this value are evaluated in the correct order
info: |
  %Iterator.prototype%.map ( mapper )

  1. Let O be the this value.
  2. If O is not an Object, throw a TypeError exception.
  3. If IsCallable(mapper) is false, throw a TypeError exception.
  4. Let iterated be ? GetIteratorDirect(O).

includes: [compareArray.js]
features: [iterator-helpers]
flags: []
---*/
let effects = [];

assert.throws(TypeError, function () {
  Iterator.prototype.map.call(
    {
      get next() {
        effects.push('get next');
        return function () {
          return { done: true, value: undefined };
        };
      },
    },
    {
      valueOf() {
        effects.push('valueOf mapper');
        return function () {
          return [];
        };
      },
    }
  );
});

assert.compareArray(effects, []);
