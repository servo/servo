// Copyright (C) 2021 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.supportedvaluesof
description: >
  The returned "timeZone" values are sorted, unique, and canonicalised.
info: |
  Intl.supportedValuesOf ( key )

  1. Let key be ? ToString(key).
  ...
  6. Else if key is "timeZone", then
    a. Let list be ! AvailableTimeZones( ).
  ...
  9. Return ! CreateArrayFromList( list ).

  AvailableTimeZones ()
    The AvailableTimeZones abstract operation returns a sorted List of supported
    Zone and Link names in the IANA Time Zone Database. The following steps are
    taken:

    1. Let names be a List of all supported Zone and Link names in the IANA Time
       Zone Database.
    2. Let result be a new empty List.
    3. For each element name of names, do
        a. Assert: ! IsValidTimeZoneName( name ) is true.
        b. Let canonical be ! CanonicalizeTimeZoneName( name ).
        c. If result does not contain an element equal to canonical, then
            i. Append canonical to the end of result.
    4. Sort result in order as if an Array of the same values had been sorted using
       %Array.prototype.sort% using undefined as comparefn.
    5. Return result.
includes: [compareArray.js, testIntl.js]
features: [Intl-enumeration]
---*/

const timeZones = Intl.supportedValuesOf("timeZone");

assert(Array.isArray(timeZones), "Returns an Array object.");
assert.sameValue(Object.getPrototypeOf(timeZones), Array.prototype,
                 "The array prototype is Array.prototype");

const otherTimeZones = Intl.supportedValuesOf("timeZone");
assert.notSameValue(otherTimeZones, timeZones,
                    "Returns a new array object for each call.");

assert.compareArray(timeZones, otherTimeZones.sort(),
                    "The array is sorted.");

assert.sameValue(new Set(timeZones).size, timeZones.length,
                 "The array doesn't contain duplicates.");

for (let timeZone of timeZones) {
  assert(isCanonicalizedStructurallyValidTimeZoneName(timeZone),
         `${timeZone} is a canonicalised and structurally valid time zone name`);
}
