// Copyright 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-availableprimarytimezoneidentifiers
description: >
  AvailablePrimaryTimeZoneIdentifiers doesn't perform any kind of filtering on
  the time zone IDs returned therefore time zone IDs for time zones that don't
  correspond to any continent like the Etc/* timezones as well as UTC should be
  included in this list.
info: |
  6.5.3 AvailablePrimaryTimeZoneIdentifiers ( )

  1. Let records be AvailableNamedTimeZoneIdentifiers().
  2. Let result be a new empty List.
  3. For each element timeZoneIdentifierRecord of records, do
    a. If timeZoneIdentifierRecord.[[Identifier]] is timeZoneIdentifierRecord.[[PrimaryIdentifier]], then
      i. Append timeZoneIdentifierRecord.[[Identifier]] to result.
  4. Return result.
features: [Intl-enumeration]
---*/

const nonContinentalTimeZones = [
  "Etc/GMT+1",
  "Etc/GMT+2",
  "Etc/GMT+3",
  "Etc/GMT+4",
  "Etc/GMT+5",
  "Etc/GMT+6",
  "Etc/GMT+7",
  "Etc/GMT+8",
  "Etc/GMT+9",
  "Etc/GMT+10",
  "Etc/GMT+11",
  "Etc/GMT+12",
  "Etc/GMT-1",
  "Etc/GMT-2",
  "Etc/GMT-3",
  "Etc/GMT-4",
  "Etc/GMT-5",
  "Etc/GMT-6",
  "Etc/GMT-7",
  "Etc/GMT-8",
  "Etc/GMT-9",
  "Etc/GMT-10",
  "Etc/GMT-11",
  "Etc/GMT-12",
  "Etc/GMT-13",
  "Etc/GMT-14",
  "UTC",
];

const supportedTimeZones = Intl.supportedValuesOf("timeZone");

for (const tz of nonContinentalTimeZones) {
  assert(
    supportedTimeZones.includes(tz),
    `non-continental timezone ${tz} is not supported`,
  );
}
