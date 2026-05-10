// Copyright 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-createdatetimeformat
description: >
  Tests that the time zone names "Etc/UTC", "Etc/GMT", and "GMT" are not
  canonicalized to "UTC" in "resolvedOptions".
info: |
  CreateDateTimeFormat ( dateTimeFormat, locales, options, required, default )

  30. If IsTimeZoneOffsetString(timeZone) is true, then
  ...
  31. Else,
    a. Let timeZoneIdentifierRecord be GetAvailableNamedTimeZoneIdentifier(timeZone).
    ...
    c. Set timeZone to timeZoneIdentifierRecord.[[Identifier]].

  GetAvailableNamedTimeZoneIdentifier ( timeZoneIdentifier )

  1. For each element record of AvailableNamedTimeZoneIdentifiers(), do
    a. If record.[[Identifier]] is an ASCII-case-insensitive match for
       timeZoneIdentifier, return record.

features: [canonical-tz]
---*/

const utcIdentifiers = ["Etc/GMT", "Etc/UTC", "GMT"];

for (const timeZone of utcIdentifiers) {
  assert.sameValue(
    new Intl.DateTimeFormat([], {timeZone}).resolvedOptions().timeZone,
    timeZone,
    "Time zone name " + timeZone + " should be preserved and not canonicalized to 'UTC'");
}
