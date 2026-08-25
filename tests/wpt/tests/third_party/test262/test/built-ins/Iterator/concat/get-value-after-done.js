// Copyright (C) 2025 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.concat
description: >
  Iterator.concat does not access the value of a done IteratorResult, diverging from the behaviour of yield*
features: [iterator-sequencing]
---*/

let valueAccesses = 0;
let iter = {
  [Symbol.iterator]() {
    return {
      next() {
        return {
          get value() {
            ++valueAccesses;
          },
          done: true,
        };
      },
    };
  }
};

Array.from(Iterator.concat(iter, iter, iter));

assert.sameValue(valueAccesses, 0, 'Iterator.concat does not access value getter after each iterator is done');
