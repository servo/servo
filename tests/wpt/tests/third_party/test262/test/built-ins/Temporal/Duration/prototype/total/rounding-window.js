// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: See https://github.com/tc39/proposal-temporal/issues/3168
features: [Temporal]
---*/

var d = new Temporal.Duration(1, 0, 0, 0, 1);
var relativeTo = new Temporal.PlainDate(2020, 2, 29);
assert.sameValue(d.total({ unit: 'years', relativeTo }), 1.0001141552511414);

d = new Temporal.Duration(0, 1, 0, 0, 10);
relativeTo = new Temporal.PlainDate(2020, 1, 31);
assert.sameValue(d.total({ unit: 'months', relativeTo }), 1.0134408602150538);
