// Copyright (C) 2021 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.supportedvaluesof
description: >
  The returned "timeZone" values can be used with DateTimeFormat.
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
locale: [en]
features: [Intl-enumeration]
---*/

const timeZones = Intl.supportedValuesOf("timeZone");

for (let timeZone of timeZones) {
  let obj = new Intl.DateTimeFormat("en", {timeZone});
  assert.sameValue(obj.resolvedOptions().timeZone, timeZone,
                   `${timeZone} is supported by DateTimeFormat`);
}
