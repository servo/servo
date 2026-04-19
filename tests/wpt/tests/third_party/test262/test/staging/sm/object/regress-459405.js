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
var BUGNUMBER = 459405;
var summary = 'Math is not ReadOnly';
var actual = '';
var expect = '';


//-----------------------------------------------------------------------------
test();
//-----------------------------------------------------------------------------

function test()
{
  expect = 'foo';

  try
  {
    var Math = 'foo';
    actual = Math;
  }
  catch(ex)
  {
    actual = ex + '';
  }

  assert.sameValue(expect, actual, summary);
}
