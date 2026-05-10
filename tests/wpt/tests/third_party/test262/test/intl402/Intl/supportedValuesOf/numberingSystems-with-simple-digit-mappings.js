// Copyright (C) 2021 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.supportedvaluesof
description: >
  The returned "numberingSystem" values contain all numbering systems with simple digit mappings.
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
includes: [testIntl.js]
features: [Intl-enumeration, Array.prototype.includes]
---*/

const numberingSystems = Intl.supportedValuesOf("numberingSystem");

// Table 10: Numbering systems with simple digit mappings
for (let numberingSystem of Object.keys(numberingSystemDigits)) {
  assert(numberingSystems.includes(numberingSystem),
         `${numberingSystem} with simple digit mappings is supported`);
}
