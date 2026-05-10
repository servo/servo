/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pobj_ == obj2
info: bugzilla.mozilla.org/show_bug.cgi?id=481516
esid: pending
---*/

var actual = '';
var expect = '';

test();

function test()
{
  expect = '1111222';

  var a = {x: 1};
  var b = {__proto__: a};
  var c = {__proto__: b};
  var objs = [{__proto__: a}, {__proto__: a}, {__proto__: a}, b, {__proto__: a},
          {__proto__: a}];
  for (var i = 0; i < 6; i++) {
    actual += ""+c.x;
    objs[i].x = 2;
  }
  actual += c.x;

  assert.sameValue(expect, actual);
}
