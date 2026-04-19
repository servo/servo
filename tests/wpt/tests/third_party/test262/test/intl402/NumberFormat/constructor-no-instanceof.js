// Copyright (C) 2021 Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl-numberformat-constructor
description: >
  Tests that the Intl.NumberFormat constructor calls
  OrdinaryHasInstance instead of the instanceof operator which includes a
  Symbol.hasInstance lookup and call among other things.
info: >
  ChainNumberFormat ( numberFormat, newTarget, this )
  1.  If newTarget is undefined and ? OrdinaryHasInstance(%NumberFormat%, this) is true, then
      a.  Perform ? DefinePropertyOrThrow(this, %Intl%.[[FallbackSymbol]], PropertyDescriptor{
          [[Value]]: numberFormat, [[Writable]]: false, [[Enumerable]]: false,
          [[Configurable]]: false }).
      b.  Return this.
---*/

Object.defineProperty(Intl.NumberFormat, Symbol.hasInstance, {
    get() { throw new Test262Error(); }
});

Intl.NumberFormat();
