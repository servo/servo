// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.from
description: >
  Check that various dates created from a RFC 9557 string have the expected
  properties
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from("2000-01-01"),
  2000, 1, "M01", 1,
  "created from string 2000-01-01");

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from("0001-01-01"),
  1, 1, "M01", 1,
  "created from string 0001-01-01");
