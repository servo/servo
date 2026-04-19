// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.from
description: >
  Iterator.from throws on primitives (except Strings)
info: |
  Iterator.from ( O )

features: [iterator-helpers]
flags: []
---*/

assert.throws(TypeError, function () {
  Iterator.from(null);
});

assert.throws(TypeError, function () {
  Iterator.from(undefined);
});

assert.throws(TypeError, function () {
  Iterator.from(0);
});

assert.throws(TypeError, function () {
  Iterator.from(0n);
});

assert.throws(TypeError, function () {
  Iterator.from(true);
});

assert.throws(TypeError, function () {
  Iterator.from(Symbol());
});

Iterator.from('string');
