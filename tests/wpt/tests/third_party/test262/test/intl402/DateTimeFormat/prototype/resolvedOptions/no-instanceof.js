// Copyright (C) 2021 Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.datetimeformat.prototype.resolvedoptions
description: >
  Tests that Intl.DateTimeFormat.prototype.resolvedOptions calls
  OrdinaryHasInstance instead of the instanceof operator which includes a
  Symbol.hasInstance lookup and call among other things.
info: >
  UnwrapDateTimeFormat ( dtf )
  2.  If dtf does not have an [[InitializedDateTimeFormat]] internal slot and
      ? OrdinaryHasInstance(%DateTimeFormat%, dtf) is true, then
      a.  Return ? Get(dtf, %Intl%.[[FallbackSymbol]]).
---*/

const dtf = Object.create(Intl.DateTimeFormat.prototype);

Object.defineProperty(Intl.DateTimeFormat, Symbol.hasInstance, {
    get() { throw new Test262Error(); }
});

assert.throws(TypeError, () => dtf.resolvedOptions());
