// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-properties-of-the-date-prototype-object
description: >
  The Date Prototype object does not have a [[DateValue]] internal slot.
info: |
  Properties of the Date Prototype Object

  The Date prototype object:
  [...]
  * is not a Date instance and does not have a [[DateValue]] internal slot.

  Date.prototype.getTime ( )

  1. Return ? thisTimeValue(this value).

  The abstract operation thisTimeValue takes argument value.

  1. If Type(value) is Object and value has a [[DateValue]] internal slot, then
    [...]
  2. Throw a TypeError exception.
---*/

assert.throws(TypeError, function() {
  Date.prototype.getTime();
});
