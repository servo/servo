// Copyright 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat
description: Property descriptor of %Intl%.[[FallbackSymbol]]
info: |
  ChainNumberFormat ( numberFormat, newTarget, this )

  1.a. Perform ? DefinePropertyOrThrow(_this_, %Intl%.[[FallbackSymbol]],
       PropertyDescriptor{ [[Value]]: _numberFormat_, [[Writable]]: *false*,
       [[Enumerable]]: *false*, [[Configurable]]: *false* }).
includes: [propertyHelper.js]
features: [intl-normative-optional]
---*/

const object = Object.create(Intl.NumberFormat.prototype);
const fallbackNF = Intl.NumberFormat.call(object);
assert.sameValue(object, fallbackNF, "return value of Intl.NumberFormat constructor");

const symbolProps = Object.getOwnPropertySymbols(fallbackNF);
const fallbackSymbol = symbolProps.find((sym) => sym.description === "IntlLegacyConstructedSymbol");

assert(
  fallbackNF[fallbackSymbol] instanceof Intl.NumberFormat,
  "value of legacy symbol property must be an Intl.NumberFormat"
);
verifyProperty(
  fallbackNF,
  fallbackSymbol,
  {
    writable: false,
    enumerable: false,
    configurable: false
  },
  { label: "%Intl%.[[FallbackSymbol]]" }
);
