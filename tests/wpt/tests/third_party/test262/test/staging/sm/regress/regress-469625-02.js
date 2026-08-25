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
var BUGNUMBER = 469625;
var summary = 'group assignment with rhs containing holes';
var actual = '';
var expect = '';


//-----------------------------------------------------------------------------
test();
//-----------------------------------------------------------------------------

function test()
{
  expect = 'y';

  Array.prototype[1] = 'y';
  var [x, y, z] = ['x', , 'z'];

  actual = y;
 
  assert.sameValue(expect, actual, summary);
}
