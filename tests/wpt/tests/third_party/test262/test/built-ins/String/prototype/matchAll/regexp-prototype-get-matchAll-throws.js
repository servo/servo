// Copyright (C) 2018 Peter Wong. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: Re-throws errors thrown while accessing RegExp's @@matchAll property
info: |
  String.prototype.matchAll ( regexp )
    [...]
    2. If regexp is neither undefined nor null, then
      a. Let matcher be ? GetMethod(regexp, @@matchAll).
features: [Symbol.matchAll]
---*/

Object.defineProperty(RegExp.prototype, Symbol.matchAll, {
  get() {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  ''.matchAll(/./g);
});
