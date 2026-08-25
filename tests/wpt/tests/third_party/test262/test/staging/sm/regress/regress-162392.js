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
// SUMMARY: 10.1.8 Arguments Object length

var BUGNUMBER = 162392;
var summary = 'eval("arguments").length == 0 when no arguments specified';
var actual = noargslength();
var expect = 0;

function noargslength()
{
  return eval('arguments').length;
}

//-----------------------------------------------------------------------------
test();
//-----------------------------------------------------------------------------

function test()
{
  assert.sameValue(expect, actual, summary);
}
