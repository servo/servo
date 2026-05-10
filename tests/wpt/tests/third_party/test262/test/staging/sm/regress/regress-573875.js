/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/

var o = {__iterator__:null, a:1, b:2, c:3}
var expect = '__iterator__,a,b,c,';
var actual = '';

for (var i in o)
    actual += i + ',';

assert.sameValue(expect, actual, "ok");
