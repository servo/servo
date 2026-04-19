// Copyright (C) 2021 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.supportedvaluesof
description: >
  The returned "numberingSystem" values are sorted, unique, and match the type production.
info: |
  Intl.supportedValuesOf ( key )

  1. Let key be ? ToString(key).
  ...
  5. Else if key is "numberingSystem", then
    a. Let list be ! AvailableNumberingSystems( ).
  ...
  9. Return ! CreateArrayFromList( list ).

  AvailableNumberingSystems ( )
    The AvailableNumberingSystems abstract operation returns a List, ordered as
    if an Array of the same values had been sorted using %Array.prototype.sort%
    using undefined as comparefn, that contains unique numbering systems
    identifiers identifying the numbering systems for which the implementation
    provides the functionality of Intl.DateTimeFormat, Intl.NumberFormat, and
    Intl.RelativeTimeFormat objects. The list must include the Numbering System
    value of every row of Table 4, except the header row.
includes: [compareArray.js]
features: [Intl-enumeration, Intl.Locale]
---*/

const numberingSystems = Intl.supportedValuesOf("numberingSystem");

assert(Array.isArray(numberingSystems), "Returns an Array object.");
assert.sameValue(Object.getPrototypeOf(numberingSystems), Array.prototype,
                 "The array prototype is Array.prototype");

const otherNumberingSystems = Intl.supportedValuesOf("numberingSystem");
assert.notSameValue(otherNumberingSystems, numberingSystems,
                    "Returns a new array object for each call.");

assert.compareArray(numberingSystems, otherNumberingSystems.sort(),
                    "The array is sorted.");

assert.sameValue(new Set(numberingSystems).size, numberingSystems.length,
                 "The array doesn't contain duplicates.");

// https://unicode.org/reports/tr35/tr35.html#Unicode_locale_identifier
const typeRE = /^[a-z0-9]{3,8}(-[a-z0-9]{3,8})*$/;
for (let numberingSystem of numberingSystems) {
  assert(typeRE.test(numberingSystem), `${numberingSystem} matches the 'type' production`);
}

for (let numberingSystem of numberingSystems) {
  assert.sameValue(new Intl.Locale("und", {numberingSystem}).numberingSystem, numberingSystem,
                   `${numberingSystem} is canonicalised`);
}
