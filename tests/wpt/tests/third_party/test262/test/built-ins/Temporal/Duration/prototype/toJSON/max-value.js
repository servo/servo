// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.tojson
description: Various maximum value combinations do not go out of range
features: [Temporal]
---*/

const maxYMW = Math.pow(2, 32) - 1; // Maximum years, months, weeks
const maxDays = Math.floor(Number.MAX_SAFE_INTEGER / 86400);
const maxHours = Math.floor(Number.MAX_SAFE_INTEGER / 3600);
const maxMinutes = Math.floor(Number.MAX_SAFE_INTEGER / 60);
const maxSecs = Number.MAX_SAFE_INTEGER; // = 9007199254740991, also max ms, μs, ns

assert.sameValue(new Temporal.Duration(maxYMW).toJSON(),
  'P' + maxYMW + 'Y', "maximum years");
assert.sameValue(new Temporal.Duration(-maxYMW).toJSON(),
  '-P' + maxYMW + 'Y', "minimum years");
assert.sameValue(new Temporal.Duration(0, maxYMW).toJSON(),
  'P' + maxYMW + 'M', "maximum months");
assert.sameValue(new Temporal.Duration(0, -maxYMW).toJSON(),
  '-P' + maxYMW + 'M', "minimum months");
assert.sameValue(new Temporal.Duration(0, 0, maxYMW).toJSON(),
  'P' + maxYMW + 'W', "maximum weeks");
assert.sameValue(new Temporal.Duration(0, 0, -maxYMW).toJSON(),
  '-P' + maxYMW + 'W', "minimum weeks");
assert.sameValue(new Temporal.Duration(0, 0, 0, maxDays).toJSON(),
  'P' + maxDays + 'D', "maximum days");
assert.sameValue(new Temporal.Duration(0, 0, 0, -maxDays).toJSON(),
  '-P' + maxDays + 'D', "minimum days");
assert.sameValue(new Temporal.Duration(0, 0, 0, 0, maxHours).toJSON(),
  'PT' + maxHours + 'H', "maximum hours");
assert.sameValue(new Temporal.Duration(0, 0, 0, 0, -maxHours).toJSON(),
  '-PT' + maxHours + 'H', "minimum hours");
assert.sameValue(new Temporal.Duration(0, 0, 0, 0, 0, maxMinutes).toJSON(),
   'PT' + maxMinutes + 'M', "maximum minutes");
assert.sameValue(new Temporal.Duration(0, 0, 0, 0, 0, -maxMinutes).toJSON(),
   '-PT' + maxMinutes + 'M', "minimum minutes");
assert.sameValue(new Temporal.Duration(0, 0, 0, 0, 0, 0, maxSecs).toJSON(),
   'PT' + maxSecs + 'S', "maximum seconds");
assert.sameValue(new Temporal.Duration(0, 0, 0, 0, 0, 0, -maxSecs).toJSON(),
   '-PT' + maxSecs + 'S', "minimum seconds");
assert.sameValue(new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, maxSecs).toJSON(),
   'PT' + Math.floor(maxSecs / 1000) + '.' + maxSecs % 1000 + 'S', "maximum milliseconds");
assert.sameValue(new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, -maxSecs).toJSON(),
   '-PT' + Math.floor(maxSecs / 1000) + '.' + maxSecs % 1000 + 'S', "minimum milliseconds");
assert.sameValue(new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, 0,  maxSecs).toJSON(),
   'PT' + Math.floor(maxSecs / 1000000) + '.' + maxSecs % 1000000 + 'S', "maximum microseconds");
assert.sameValue(new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, 0, -maxSecs).toJSON(),
   '-PT' + Math.floor(maxSecs / 1000000) + '.' + maxSecs % 1000000 + 'S', "minimum microseconds");
assert.sameValue(new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, 0, 0, maxSecs).toJSON(),
   'PT' + Math.floor(maxSecs / 1000000000) + '.' + maxSecs % 1000000000 + 'S', "maximum nanoseconds");
assert.sameValue(new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, 0, 0, -maxSecs).toJSON(),
   '-PT' + Math.floor(maxSecs / 1000000000) + '.' + maxSecs % 1000000000 + 'S', "minimum nanoseconds");

// Combinations with maximum values without balancing.
assert.sameValue(new Temporal.Duration(0, 0, 0, 0, 0, 0, maxSecs, 999).toJSON(),
  'PT' + maxSecs + '.999S', "max value ms and s does not go out of range");
assert.sameValue(new Temporal.Duration(0, 0, 0, 0, 0, 0, -maxSecs, -999).toJSON(),
  '-PT' + maxSecs + '.999S', "min value ms and s does not go out of range");
assert.sameValue(new Temporal.Duration(0, 0, 0, 0, 0, 0, maxSecs, 999, 999).toJSON(),
  'PT' + maxSecs + '.999999S', "max value ms, μs and s does not go out of range");
assert.sameValue(new Temporal.Duration(0, 0, 0, 0, 0, 0, -maxSecs, -999, -999).toJSON(),
  '-PT' + maxSecs + '.999999S', "min value ms, μs and s does not go out of range");
assert.sameValue(new Temporal.Duration(0, 0, 0, 0, 0, 0, maxSecs, 999, 999, 999).toJSON(),
  'PT' + maxSecs + '.999999999S', "max value ms, μs, ns and s does not go out of range");
assert.sameValue(new Temporal.Duration(0, 0, 0, 0, 0, 0, -maxSecs, -999, -999, -999).toJSON(),
  '-PT' + maxSecs + '.999999999S', "min value ms, μs, ns and s does not go out of range");

// Combinations with maximum values with balancing.
const balanceMaxSecondsMilliseconds = new Temporal.Duration(
  0, 0, 0, 0, 0, 0, maxSecs - Math.floor(maxSecs / 1000), maxSecs, 0, 0);
assert.sameValue(balanceMaxSecondsMilliseconds.toJSON(),
  'PT' + maxSecs + '.' + maxSecs % 1000 + 'S', "balancing max ms to max s");
const balanceMinSecondsMilliseconds = new Temporal.Duration(
  0, 0, 0, 0, 0, 0, -(maxSecs - Math.floor(maxSecs / 1000)), -maxSecs, 0, 0);
assert.sameValue(balanceMinSecondsMilliseconds.toJSON(),
  '-PT' + maxSecs + '.' + maxSecs % 1000 + 'S', "balancing min ms to max s");
const balanceMaxSecondsMicroseconds = new Temporal.Duration(
  0, 0, 0, 0, 0, 0, maxSecs - Math.floor(maxSecs / 1000000), 0, maxSecs, 0);
assert.sameValue(balanceMaxSecondsMicroseconds.toJSON(),
  'PT' + maxSecs + '.' + maxSecs % 1000000 + 'S', "balancing max μs to max s");
const balanceMinSecondsMicroseconds = new Temporal.Duration(
  0, 0, 0, 0, 0, 0, -(maxSecs - Math.floor(maxSecs / 1000000)), 0, -maxSecs, 0);
assert.sameValue(balanceMinSecondsMicroseconds.toJSON(),
  '-PT' + maxSecs + '.' + maxSecs % 1000000 + 'S', "balancing min μs to min s");
const balanceMaxSecondsNanoseconds = new Temporal.Duration(
  0, 0, 0, 0, 0, 0, maxSecs - Math.floor(maxSecs / 1000000000), 0, 0, maxSecs);
assert.sameValue(balanceMaxSecondsNanoseconds.toJSON(),
  'PT' + maxSecs + '.' + maxSecs % 1000000000 + 'S', "balancing max ns to max s");
const balanceMinSecondsNanoseconds = new Temporal.Duration(
  0, 0, 0, 0, 0, 0, -(maxSecs - Math.floor(maxSecs / 1000000000)), 0, 0, -maxSecs);
assert.sameValue(balanceMinSecondsNanoseconds.toJSON(),
  '-PT' + maxSecs + '.' + maxSecs % 1000000000 + 'S', "balancing min ns to min s");
