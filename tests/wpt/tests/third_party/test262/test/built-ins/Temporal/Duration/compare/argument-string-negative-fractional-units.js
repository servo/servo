// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.compare
description: Strings with fractional duration units are treated with the correct sign
features: [Temporal]
---*/

const expectedHours = new Temporal.Duration(0, 0, 0, 0, -24, -34, -4, -404, -442, -800);
const resultHours1 = Temporal.Duration.compare("-PT24.567890123H", expectedHours);
assert.sameValue(resultHours1, 0, "negative fractional hours (first argument)");
const resultHours2 = Temporal.Duration.compare(expectedHours, "-PT24.567890123H");
assert.sameValue(resultHours2, 0, "negative fractional hours (second argument)");

const expectedMinutes = new Temporal.Duration(0, 0, 0, 0, 0, -1440, -34, -73, -407, -380);
const resultMinutes1 = Temporal.Duration.compare("-PT1440.567890123M", expectedMinutes);
assert.sameValue(resultMinutes1, 0, "negative fractional minutes (first argument)");
const resultMinutes2 = Temporal.Duration.compare("-PT1440.567890123M", expectedMinutes);
assert.sameValue(resultMinutes2, 0, "negative fractional minutes (second argument)");
