/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Do not assert: pobj_ == obj2
info: bugzilla.mozilla.org/show_bug.cgi?id=480579
esid: pending
---*/

var actual = '';
var expect = '';

test();

function test()
{
  expect = '12';

  var a = {x: 1};
  var b = {__proto__: a};
  var c = {__proto__: b};
  for (var i = 0; i < 2; i++) {
    actual += c.x;
    b.x = 2;
  }

  assert.sameValue(expect, actual);
}
