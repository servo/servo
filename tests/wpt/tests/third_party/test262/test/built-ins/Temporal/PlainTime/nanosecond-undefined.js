// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime
description: Nanosecond argument defaults to 0 if not given
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const args = [12, 34, 56, 123, 456];

const explicit = new Temporal.PlainTime(...args, undefined);
TemporalHelpers.assertPlainTime(explicit, ...args, 0, "explicit");

const implicit = new Temporal.PlainTime(...args);
TemporalHelpers.assertPlainTime(implicit, ...args, 0, "implicit");
