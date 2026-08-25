// Copyright (C) 2021 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.supportedvaluesof
description: >
  The returned "currency" values are sorted, unique, and upper-case canonicalised.
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
includes: [compareArray.js]
features: [Intl-enumeration]
---*/

const currencies = Intl.supportedValuesOf("currency");

assert(Array.isArray(currencies), "Returns an Array object.");
assert.sameValue(Object.getPrototypeOf(currencies), Array.prototype,
                 "The array prototype is Array.prototype");

const otherCurrencies = Intl.supportedValuesOf("currency");
assert.notSameValue(otherCurrencies, currencies,
                    "Returns a new array object for each call.");

assert.compareArray(currencies, otherCurrencies.sort(),
                    "The array is sorted.");

assert.sameValue(new Set(currencies).size, currencies.length,
                 "The array doesn't contain duplicates.");

const codeRE = /^[A-Z]{3}$/;
for (let currency of currencies) {
  assert(codeRE.test(currency), `${currency} is a 3-letter ISO 4217 currency code`);
}
