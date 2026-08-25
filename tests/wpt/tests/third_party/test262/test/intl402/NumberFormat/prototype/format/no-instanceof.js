// Copyright (C) 2021 Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat.prototype.format
description: >
  Tests that Intl.NumberFormat.prototype.format calls
  OrdinaryHasInstance instead of the instanceof operator which includes a
  Symbol.hasInstance lookup and call among other things.
info: >
  UnwrapNumberFormat ( nf )

  2.  If nf does not have an [[InitializedNumberFormat]] internal slot and
      ? OrdinaryHasInstance(%NumberFormat%, nf) is true, then
      a.  Return ? Get(nf, %Intl%.[[FallbackSymbol]]).
---*/

const nf = Object.create(Intl.NumberFormat.prototype);

Object.defineProperty(Intl.NumberFormat, Symbol.hasInstance, {
    get() { throw new Test262Error(); }
});

assert.throws(TypeError, () => nf.format);
