// Copyright (C) 2021 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.supportedvaluesof
description: >
  The returned "unit" values can be used with NumberFormat.
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
includes: [testIntl.js]
locale: [en]
features: [Intl-enumeration, Array.prototype.includes]
---*/

const units = Intl.supportedValuesOf("unit");

for (let unit of units) {
  let obj = new Intl.NumberFormat("en", {style: "unit", unit});
  assert.sameValue(obj.resolvedOptions().unit, unit,
                   `${unit} is supported by NumberFormat`);
}

for (let unit of allSimpleSanctionedUnits()) {
  let obj = new Intl.NumberFormat("en", {style: "unit", unit});
  if (obj.resolvedOptions().unit === unit) {
    assert(units.includes(unit),
           `${unit} supported but not returned by supportedValuesOf`);
  } else {
    assert(!units.includes(unit),
           `${unit} not supported but returned by supportedValuesOf`);
  }
}
