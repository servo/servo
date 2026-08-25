// Copyright (C) 2021 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.supportedvaluesof
description: >
  The returned "currency" values can be used with DisplayNames.
info: |
  Intl.supportedValuesOf ( key )

  1. Let key be ? ToString(key).
  ...
  4. Else if key is "currency", then
    a. Let list be ! AvailableCurrencies( ).
  ...
  9. Return ! CreateArrayFromList( list ).

  AvailableCurrencies ( )
    The AvailableCurrencies abstract operation returns a List, ordered as if an
    Array of the same values had been sorted using %Array.prototype.sort% using
    undefined as comparefn, that contains unique, well-formed, and upper case
    canonicalized 3-letter ISO 4217 currency codes, identifying the currencies
    for which the implementation provides the functionality of Intl.DisplayNames
    and Intl.NumberFormat objects.
locale: [en]
features: [Intl-enumeration, Intl.DisplayNames, Array.prototype.includes]
---*/

const currencies = Intl.supportedValuesOf("currency");

const obj = new Intl.DisplayNames("en", {type: "currency", fallback: "none"});

for (let currency of currencies) {
  assert.sameValue(typeof obj.of(currency), "string",
                   `${currency} is supported by DisplayNames`);
}

for (let i = 0x41; i <= 0x5A; ++i) {
  for (let j = 0x41; j <= 0x5A; ++j) {
    for (let k = 0x41; k <= 0x5A; ++k) {
      let currency = String.fromCharCode(i, j, k);
      if (typeof obj.of(currency) === "string") {
        assert(currencies.includes(currency),
               `${currency} supported but not returned by supportedValuesOf`);
      } else {
        assert(!currencies.includes(currency),
               `${currency} not supported but returned by supportedValuesOf`);
      }
    }
  }
}
