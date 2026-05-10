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
var BUGNUMBER = 441477;
var summary = '';
var actual = 'No Exception';
var expect = 'No Exception';


//-----------------------------------------------------------------------------
test();
//-----------------------------------------------------------------------------

function test()
{
  try
  {
    for (var i = 0; i < 5;)
    {
      if (i > 5)
        throw "bad";
      i++;
      continue;
    }
  }
  catch(ex)
  {
    actual = ex + '';
  }
  assert.sameValue(expect, actual, summary);
}
