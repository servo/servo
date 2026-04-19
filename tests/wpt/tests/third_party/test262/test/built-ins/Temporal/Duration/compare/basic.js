// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.compare
description: Basic comparisons.
features: [Temporal]
---*/

const td1pos = new Temporal.Duration(0, 0, 0, 0, 5, 5, 5, 5, 5, 5);
const td2pos = new Temporal.Duration(0, 0, 0, 0, 5, 4, 5, 5, 5, 5);
const td1neg = new Temporal.Duration(0, 0, 0, 0, -5, -5, -5, -5, -5, -5);
const td2neg = new Temporal.Duration(0, 0, 0, 0, -5, -4, -5, -5, -5, -5);
assert.sameValue(Temporal.Duration.compare(td1pos, td1pos), 0,
  "time units: equal");
assert.sameValue(Temporal.Duration.compare(td2pos, td1pos), -1,
  "time units: smaller/larger");
assert.sameValue(Temporal.Duration.compare(td1pos, td2pos), 1,
  "time units: larger/smaller");
assert.sameValue(Temporal.Duration.compare(td1neg, td1neg), 0,
  "time units: negative/negative equal");
assert.sameValue(Temporal.Duration.compare(td2neg, td1neg), 1,
  "time units: negative/negative smaller/larger");
assert.sameValue(Temporal.Duration.compare(td1neg, td2neg), -1,
  "time units: negative/negative larger/smaller");
assert.sameValue(Temporal.Duration.compare(td1neg, td2pos), -1,
  "time units: negative/positive");
assert.sameValue(Temporal.Duration.compare(td1pos, td2neg), 1,
  "time units: positive/negative");

const dd1pos = new Temporal.Duration(5, 5, 5, 5, 5, 5, 5, 5, 5, 5);
const dd2pos = new Temporal.Duration(5, 5, 5, 5, 5, 4, 5, 5, 5, 5);
const dd1neg = new Temporal.Duration(-5, -5, -5, -5, -5, -5, -5, -5, -5, -5);
const dd2neg = new Temporal.Duration(-5, -5, -5, -5, -5, -4, -5, -5, -5, -5);
const relativeTo = Temporal.PlainDate.from("2017-01-01");
assert.throws(RangeError, () => Temporal.Duration.compare(dd1pos, dd2pos),
  "date units: relativeTo is required");
assert.sameValue(Temporal.Duration.compare(dd1pos, dd1pos, { relativeTo }), 0,
  "date units: equal");
assert.sameValue(Temporal.Duration.compare(dd2pos, dd1pos, { relativeTo }), -1,
  "date units: smaller/larger");
assert.sameValue(Temporal.Duration.compare(dd1pos, dd2pos, { relativeTo }), 1,
  "date units: larger/smaller");
assert.sameValue(Temporal.Duration.compare(dd1neg, dd1neg, { relativeTo }), 0,
  "date units: negative/negative equal");
assert.sameValue(Temporal.Duration.compare(dd2neg, dd1neg, { relativeTo }), 1,
  "date units: negative/negative smaller/larger");
assert.sameValue(Temporal.Duration.compare(dd1neg, dd2neg, { relativeTo }), -1,
  "date units: negative/negative larger/smaller");
assert.sameValue(Temporal.Duration.compare(dd1neg, dd2pos, { relativeTo }), -1,
  "date units: negative/positive");
assert.sameValue(Temporal.Duration.compare(dd1pos, dd2neg, { relativeTo }), 1,
  "date units: positive/negative");
