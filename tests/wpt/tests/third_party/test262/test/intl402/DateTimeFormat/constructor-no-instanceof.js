// Copyright (C) 2021 Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl-datetimeformat-constructor
description: >
  Tests that the Intl.DateTimeFormat constructor calls
  OrdinaryHasInstance instead of the instanceof operator which includes a
  Symbol.hasInstance lookup and call among other things.
info: >
  ChainDateTimeFormat ( dateTimeFormat, newTarget, this )
  1.  If newTarget is undefined and ? OrdinaryHasInstance(%DateTimeFormat%, this) is true, then
      a.  Perform ? DefinePropertyOrThrow(this, %Intl%.[[FallbackSymbol]], PropertyDescriptor{
          [[Value]]: dateTimeFormat, [[Writable]]: false, [[Enumerable]]: false,
          [[Configurable]]: false }).
      b.  Return this.
---*/

Object.defineProperty(Intl.DateTimeFormat, Symbol.hasInstance, {
    get() { throw new Test262Error(); }
});

Intl.DateTimeFormat();
