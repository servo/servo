// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.from
description: overflow property is not extracted with ISO-invalid string argument.
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

let actual = [];
const options = TemporalHelpers.propertyBagObserver(actual, { overflow: "reject" }, "options");

assert.throws(RangeError, () => Temporal.PlainDateTime.from("2020-13-34T25:60:60", options));
assert.compareArray(actual, [], "options read after string parsing");
