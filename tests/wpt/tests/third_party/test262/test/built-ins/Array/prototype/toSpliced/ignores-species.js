// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.tospliced
description: >
  Array.prototype.toSpliced ignores @@species
info: |
  Array.prototype.toSpliced ( start, deleteCount, ...items )

  ...
  12. Let A be ? ArrayCreate(𝔽(newLen)).
  ...
features: [change-array-by-copy]
---*/

var a = [];
a.constructor = {};
a.constructor[Symbol.species] = function () {}

assert.sameValue(Object.getPrototypeOf(a.toSpliced(0, 0)), Array.prototype);

var b = [];
Object.defineProperty(b, "constructor", {
  get() {
    throw new Test262Error("Should not get .constructor");
  }
});

b.toSpliced(0, 0);
