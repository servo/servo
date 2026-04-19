// Copyright (C) 2025 Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.supportedvaluesof
description: >
  Verifies that all calendars required by Intl.Era-monthcode are supported. See:
  https://tc39.es/proposal-intl-era-monthcode/#table-calendar-types
info: |
  Intl.supportedValuesOf ( key )
    1  Let key be ? ToString(key).
    2. If key is "calendar", then
     a. Let list be a new empty List.
     b. For each element identifier of AvailableCalendars(), do
        i. Let canonical be CanonicalizeUValue("ca", identifier).
        ii. If identifier is canonical, then
          1. Append identifier to list.
          ...
    9. Return CreateArrayFromList( list ).

  AvailableCalendars ( )
    The implementation-defined abstract operation AvailableCalendars takes no arguments and returns
    a List of calendar types. The returned List is sorted according to lexicographic code unit order,
    and contains unique calendar types in canonical form (6.9) identifying the calendars for which
    the implementation provides the functionality of Intl.DateTimeFormat objects, including their
    aliases (e.g., both "islamicc" and "islamic-civil"). The List must consist
    of the Calendar Type value of every row of Table 1, except the header row.
features: [Intl-enumeration, Intl.Era-monthcode]
---*/

const requiredCalendars = [
  "buddhist",
  "chinese",
  "coptic",
  "dangi",
  "ethioaa",
  "ethiopic",
  // "ethiopic-amete-alem", // Alias for "ethioaa".
  "gregory",
  "hebrew",
  "indian",
  "islamic-civil",
  "islamic-tbla",
  "islamic-umalqura",
  // "islamicc", // Alias for "islamic-civil".
  "iso8601",
  "japanese",
  "persian",
  "roc"
];

assert.compareArray(
  requiredCalendars,
  requiredCalendars.slice(0).sort(),
  "requiredCalendars is sorted"
);

const supportedCalendars = Intl.supportedValuesOf("calendar");

assert.compareArray(
  supportedCalendars,
  requiredCalendars,
  "only the required calendars are supported"
);
