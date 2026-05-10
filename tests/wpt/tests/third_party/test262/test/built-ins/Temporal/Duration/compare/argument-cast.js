// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.compare
description: Strings and objects are supported arguments.
features: [Temporal]
---*/

assert.sameValue(Temporal.Duration.compare("PT12H", new Temporal.Duration()), 1,
  "first argument string");
assert.sameValue(Temporal.Duration.compare({ hours: 12 }, new Temporal.Duration()), 1,
  "first argument object");
assert.throws(TypeError, () => Temporal.Duration.compare({ hour: 12 }, new Temporal.Duration()),
  "first argument missing property");

assert.sameValue(Temporal.Duration.compare(new Temporal.Duration(), "PT12H"), -1,
  "second argument string");
assert.sameValue(Temporal.Duration.compare(new Temporal.Duration(), { hours: 12 }), -1,
  "second argument object");
assert.throws(TypeError, () => Temporal.Duration.compare(new Temporal.Duration(), { hour: 12 }),
  "second argument missing property");

assert.sameValue(Temporal.Duration.compare({ hours: 12, minute: 5 }, { hours: 12, day: 5 }), 0,
  "ignores incorrect properties");

