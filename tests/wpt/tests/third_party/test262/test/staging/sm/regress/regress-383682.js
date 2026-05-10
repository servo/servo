/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
//-----------------------------------------------------------------------------
var BUGNUMBER = 383682;
var summary = 'eval is too dynamic';
var actual = '';
var expect = '';


//-----------------------------------------------------------------------------
test();
//-----------------------------------------------------------------------------

function test()
{
  function f(s) {
    return this.eval(s);
  }

  expect = 'PASS';
  f("function g() { return('PASS'); }");
  actual = g();

  assert.sameValue(expect, actual, summary);
}
