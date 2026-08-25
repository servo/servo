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
var BUGNUMBER = 459293;
var summary = 'Allow redeclaration of JSON';
var actual = '';
var expect = '';
 
  try
  {
    eval('var JSON = "foo";');
  }
  catch(ex)
  {
    actual = ex + '';
  }
  assert.sameValue(expect, actual, summary);
