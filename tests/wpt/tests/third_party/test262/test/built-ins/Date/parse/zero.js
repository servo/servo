// Copyright (C) 2018 Andrew Paprocki. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date.parse
description: >
  Date.parse of toString/toUTCString/toISOString of zero value is zero
info: |
  Date.parse ( string )

  If x is any Date object whose milliseconds amount is zero within a
  particular implementation of ECMAScript, then all of the following
  expressions should produce the same numeric value in that
  implementation, if all the properties referenced have their initial
  values:

  x.valueOf()
  Date.parse(x.toString())
  Date.parse(x.toUTCString())
  Date.parse(x.toISOString())
---*/

const zero = new Date(0);

assert.sameValue(zero.valueOf(), Date.parse(zero.toString()),
                 "Date.parse(zeroDate.toString())");
assert.sameValue(zero.valueOf(), Date.parse(zero.toUTCString()),
                 "Date.parse(zeroDate.toUTCString())");
assert.sameValue(zero.valueOf(), Date.parse(zero.toISOString()),
                 "Date.parse(zeroDate.toISOString())");
