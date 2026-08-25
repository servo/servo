// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: >
  Check that various dates created from a RFC 9557 string have the expected
  properties
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from("2000-01-01T12:34:56.987654321Z[UTC]").toPlainDateTime(),
  2000, 1, "M01", 1, 12, 34, 56, 987, 654, 321,
  "created from string 2000-01-01T12:34:56.987654321Z[UTC]");

TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from("0001-01-01T12:34:56.987654321Z[UTC]").toPlainDateTime(),
  1, 1, "M01", 1, 12, 34, 56, 987, 654, 321,
  "created from string 0001-01-01T12:34:56.987654321Z[UTC]");
