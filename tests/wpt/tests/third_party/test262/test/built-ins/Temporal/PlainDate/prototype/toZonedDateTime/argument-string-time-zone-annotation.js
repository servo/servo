// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.tozoneddatetime
description: Various forms of time zone annotation; critical flag has no effect
features: [Temporal]
---*/

const tests = [
  ["12:34:56.987654321[Asia/Kolkata]", "named, with no offset"],
  ["12:34:56.987654321[!Europe/Vienna]", "named, with ! and no offset"],
  ["12:34:56.987654321[+00:00]", "numeric, with no offset"],
  ["12:34:56.987654321[!-02:30]", "numeric, with ! and no offset"],
  ["T12:34:56.987654321[UTC]", "named, with T and no offset"],
  ["T12:34:56.987654321[!Africa/Abidjan]", "named, with T, !, and no offset"],
  ["T12:34:56.987654321[+01:00]", "numeric, with T and no offset"],
  ["T12:34:56.987654321[!-08:00]", "numeric, with T, !, and no offset"],
  ["12:34:56.987654321+00:00[America/Sao_Paulo]", "named, with offset"],
  ["12:34:56.987654321+00:00[!Asia/Tokyo]", "named, with ! and offset"],
  ["12:34:56.987654321+00:00[-02:30]", "numeric, with offset"],
  ["12:34:56.987654321+00:00[!+00:00]", "numeric, with ! and offset"],
  ["T12:34:56.987654321+00:00[America/New_York]", "named, with T and offset"],
  ["T12:34:56.987654321+00:00[!UTC]", "named, with T, !, and offset"],
  ["T12:34:56.987654321+00:00[-08:00]", "numeric, with T and offset"],
  ["T12:34:56.987654321+00:00[!+01:00]", "numeric, with T, !, and offset"],
  ["1970-01-01T12:34:56.987654321[Africa/Lagos]", "named, with date and no offset"],
  ["1970-01-01T12:34:56.987654321[!America/Vancouver]", "named, with date, !, and no offset"],
  ["1970-01-01T12:34:56.987654321[+00:00]", "numeric, with date and no offset"],
  ["1970-01-01T12:34:56.987654321[!-02:30]", "numeric, with date, !, and no offset"],
  ["1970-01-01T12:34:56.987654321+00:00[Europe/London]", "named, with date and offset"],
  ["1970-01-01T12:34:56.987654321+00:00[!Asia/Seoul]", "named, with date, offset, and !"],
  ["1970-01-01T12:34:56.987654321+00:00[+01:00]", "numeric, with date and offset"],
  ["1970-01-01T12:34:56.987654321+00:00[!-08:00]", "numeric, with date, offset, and !"],
];

const instance = new Temporal.PlainDate(2000, 5, 2);

tests.forEach(([arg, description]) => {
  const result = instance.toZonedDateTime({ plainTime: arg, timeZone: "UTC" });

  assert.sameValue(
    result.epochNanoseconds,
    957_270_896_987_654_321n,
    `time zone annotation (${description})`
  );
});
