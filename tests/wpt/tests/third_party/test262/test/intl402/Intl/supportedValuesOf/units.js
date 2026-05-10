// Copyright (C) 2021 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.supportedvaluesof
description: >
  The returned "unit" values are sorted, unique, and well-formed.
info: |
  Intl.supportedValuesOf ( key )

  1. Let key be ? ToString(key).
  ...
  7. Else if key is "unit", then
    a. Let list be ! AvailableUnits( ).
  ...
  9. Return ! CreateArrayFromList( list ).

  AvailableUnits ( )
    The AvailableUnits abstract operation returns a List, ordered as if an Array
    of the same values had been sorted using %Array.prototype.sort% using
    undefined as comparefn, that contains the unique values of simple unit
    identifiers listed in every row of Table 1, except the header row.
includes: [compareArray.js, testIntl.js]
features: [Intl-enumeration, Array.prototype.includes]
---*/

const units = Intl.supportedValuesOf("unit");

assert(Array.isArray(units), "Returns an Array object.");
assert.sameValue(Object.getPrototypeOf(units), Array.prototype,
                 "The array prototype is Array.prototype");

const otherUnits = Intl.supportedValuesOf("unit");
assert.notSameValue(otherUnits, units,
                    "Returns a new array object for each call.");

assert.compareArray(units, otherUnits.sort(),
                    "The array is sorted.");

assert.sameValue(new Set(units).size, units.length,
                 "The array doesn't contain duplicates.");

const simpleSanctioned = allSimpleSanctionedUnits();

for (let unit of units) {
  assert(simpleSanctioned.includes(unit), `${unit} is a simple, sanctioned unit`);
  assert(!unit.includes("-per-"), `${unit} isn't a compound unit`);
}
