// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date.prototype.tostring
description: Date.prototype.toString throws a TypeError on non-Date receivers
info: |
  Date.prototype.toString ( )

  1. Let tv be ? thisTimeValue(this value).
---*/

assert.throws(TypeError, () => Date.prototype.toString());
assert.throws(TypeError, () => Date.prototype.toString.call(undefined));
assert.throws(TypeError, () => Date.prototype.toString.call(0));
assert.throws(TypeError, () => Date.prototype.toString.call({}));
assert.throws(TypeError, () =>
  Date.prototype.toString.call("Tue Mar 21 2017 12:16:43 GMT-0400 (EDT)"));
assert.throws(TypeError, () =>
  Date.prototype.toString.call(1490113011493));
