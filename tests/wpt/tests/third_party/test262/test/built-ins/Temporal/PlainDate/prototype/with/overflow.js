// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.with
description: Basic tests for the overflow option
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const date = new Temporal.PlainDate(1976, 11, 18);

assert.throws(
  RangeError,
  () => date.with({ year: -300000 }),
  "too-low year rejects even with overflow constrain");

assert.throws(
  RangeError,
  () => date.with({ year: 300000 }),
  "too-high year rejects even with overflow constrain");

assert.throws(
  RangeError,
  () => date.with({ month: 0 }),
  "non-positive month rejects even with overflow constrain");

TemporalHelpers.assertPlainDate(
  date.with({ month: 13 }),
  1976, 12, "M12", 18,
  "too-high month is constrained to highest value");

assert.throws(RangeError, function () {
  date.with({ month: 13 }, { overflow: "reject" });
}, "too-high month rejects");

assert.throws(
  RangeError,
  () => date.with({ monthCode: "M13" }),
  "Invalid monthCode for calendar rejects even with overflow constrain");

assert.throws(
  RangeError,
  () => date.with({ day: 0 }),
  "non-positive day rejects even with overflow constrain");

TemporalHelpers.assertPlainDate(
  date.with({ day: 31 }),
  1976, 11, "M11", 30,
  "too-high day is constrained to highest value");

assert.throws(RangeError, function () {
  date.with({ day: 31 }, { overflow: "reject" });
}, "too-high day rejects");

assert.throws(
  RangeError,
  () => date.with({ month: 5, monthCode: "M06" }),
  "Conflicting month and monthCode rejects even with overflow constrain");
