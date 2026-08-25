/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
//-----------------------------------------------------------------------------
var BUGNUMBER = 566651;
var summary = 'setting array.length to null should not throw an uncatchable exception';
var actual = 0;
var expect = 0;

var a = [];
a.length = null;

assert.sameValue(expect, actual, summary);
