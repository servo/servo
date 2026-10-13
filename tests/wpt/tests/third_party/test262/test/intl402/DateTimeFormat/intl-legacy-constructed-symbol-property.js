// Copyright 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.datetimeformat
description: Property descriptor of %Intl%.[[FallbackSymbol]]
info: |
  ChainDateTimeFormat ( dateTimeFormat, newTarget, this )

  1.a. Perform ? DefinePropertyOrThrow(_this_, %Intl%.[[FallbackSymbol]],
       PropertyDescriptor{ [[Value]]: _dateTimeFormat_, [[Writable]]: *false*,
       [[Enumerable]]: *false*, [[Configurable]]: *false* }).
includes: [propertyHelper.js]
features: [intl-normative-optional]
---*/

const object = Object.create(Intl.DateTimeFormat.prototype);
const fallbackDTF = Intl.DateTimeFormat.call(object);
assert.sameValue(object, fallbackDTF, "return value of Intl.DateTimeFormat constructor");

const symbolProps = Object.getOwnPropertySymbols(fallbackDTF);
const fallbackSymbol = symbolProps.find((sym) => sym.description === "IntlLegacyConstructedSymbol");

assert(
  fallbackDTF[fallbackSymbol] instanceof Intl.DateTimeFormat,
  "value of legacy symbol property must be an Intl.DateTimeFormat"
);
verifyProperty(
  fallbackDTF,
  fallbackSymbol,
  {
    writable: false,
    enumerable: false,
    configurable: false
  },
  { label: "%Intl%.[[FallbackSymbol]]" }
);
