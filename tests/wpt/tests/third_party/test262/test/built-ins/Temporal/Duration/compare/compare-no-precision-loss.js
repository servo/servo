// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.compare
description: Temporal.Duration.compare() does not lose precision when totaling everything down to nanoseconds.
features: [Temporal]
---*/

const days200 = new Temporal.Duration(0, 0, 0, 200);
const days200oneNanosecond = new Temporal.Duration(0, 0, 0, 200, 0, 0, 0, 0, 0, 1);

assert.notSameValue(Temporal.Duration.compare(days200, days200oneNanosecond), 0);
