// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat.prototype.format
description: >
  "format" basic tests for invalid arguments that should throw TypeError exception.
info: |
  Intl.DurationFormat.prototype.format(duration)
  (...)
  3. Let record be ? ToDurationRecord(duration)
features: [Intl.DurationFormat]
---*/

const df = new Intl.DurationFormat();

assert.throws(TypeError, () => { df.format(undefined) }, "undefined" );
assert.throws(TypeError, () => { df.format(null) }, "null");
assert.throws(TypeError, () => { df.format(true) }, "true");
assert.throws(TypeError, () => { df.format(-12) }, "-12");
assert.throws(TypeError, () => { df.format(-12n) }, "-12n");
assert.throws(TypeError, () => { df.format(1) }, "1");
assert.throws(TypeError, () => { df.format(2n) }, "2n");
assert.throws(TypeError, () => { df.format({}) }, "plain object");
assert.throws(TypeError, () => { df.format({ year: 1 }) }, "unsuported property");
assert.throws(TypeError, () => { df.format({ years: undefined }) }, "supported property set undefined");
assert.throws(TypeError, () => { df.format(Symbol())}, "symbol");
assert.throws(RangeError, () => { df.format("bad string")}, "bad string");
