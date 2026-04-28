// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.reduce
description: >
  Arguments and this value are evaluated in the correct order
info: |
  %Iterator.prototype%.reduce ( reducer )

includes: [compareArray.js]
features: [iterator-helpers]
flags: []
---*/
let effects = [];

assert.throws(TypeError, function () {
  Iterator.prototype.reduce.call(
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
        effects.push('reducer valueOf');
      }
    },
    {
      valueOf() {
        effects.push('initial value valueOf');
      }
    }
  );
});

assert.compareArray(effects, []);

Iterator.prototype.reduce.call(
  {
    get next() {
      effects.push('get next');
      return function () {
        return { done: true, value: undefined };
      };
    },
  },
  () => {},
  {
    valueOf() {
      effects.push('initial value valueOf');
    }
  }
);

assert.compareArray(effects, ['get next']);
