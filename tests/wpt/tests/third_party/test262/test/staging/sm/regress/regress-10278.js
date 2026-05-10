/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
/**
 *  File Name:          regress-10278.js
 *  Reference:          https://bugzilla.mozilla.org/show_bug.cgi?id=10278
 *  Description:        Function declarations do not need to be separated
 *                      by semi-colon if they occur on the same line.
 *  Author:             bob@bclary.com
 */
//-----------------------------------------------------------------------------
var BUGNUMBER = 10278;
var summary = 'Function declarations do not need to be separated by semi-colon';
var actual;
var expect;


//-----------------------------------------------------------------------------
test();
//-----------------------------------------------------------------------------

function test()
{
  expect = 'pass';
  try
  {
    eval("function f(){}function g(){}");
    actual = "pass";
  }
  catch ( e )
  {
    actual = "fail";
  }

  assert.sameValue(expect, actual, summary);
}
