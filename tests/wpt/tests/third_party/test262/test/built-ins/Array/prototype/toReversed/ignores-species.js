// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.toreversed
description: >
  Array.prototype.toReversed ignores @@species
info: |
  Array.prototype.toReversed ( )

  ...
  3. Let A be ? ArrayCreate(𝔽(len)).
  ...
features: [change-array-by-copy]
---*/

var a = [];
a.constructor = {};
a.constructor[Symbol.species] = function () {}

assert.sameValue(Object.getPrototypeOf(a.toReversed()), Array.prototype);

var b = [];

Object.defineProperty(b, "constructor", {
  get() {
    throw new Test262Error("Should not get .constructor");
  }
});

b.toReversed();
