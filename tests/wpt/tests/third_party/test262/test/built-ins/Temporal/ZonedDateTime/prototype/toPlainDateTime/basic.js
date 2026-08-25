// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.toplaindatetime
description: Test of basic functionality
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const zdtEpoch = new Temporal.ZonedDateTime(0n, "UTC");
TemporalHelpers.assertPlainDateTime(
  zdtEpoch.toPlainDateTime(),
  1970, 1, "M01", 1, 0, 0, 0, 0, 0, 0,
  "epoch result"
);

const zdtPostEpoch = Temporal.ZonedDateTime.from("2014-11-21T14:32:27.234567891[+01:00]");
TemporalHelpers.assertPlainDateTime(
  zdtPostEpoch.toPlainDateTime(),
  2014, 11, "M11", 21, 14, 32, 27, 234, 567, 891,
  "post-epoch result"
);

const zdtPreEpoch = Temporal.ZonedDateTime.from("1969-07-16T13:32:01.234567891Z[-04:00]");
TemporalHelpers.assertPlainDateTime(
  zdtPreEpoch.toPlainDateTime(),
  1969, 7, "M07", 16, 9, 32, 1, 234, 567, 891,
  "pre-epoch result"
);
