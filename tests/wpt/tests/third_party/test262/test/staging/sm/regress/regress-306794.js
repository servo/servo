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
var BUGNUMBER = 306794;
var summary = 'Do not assert: parsing foo getter';
var actual = 'No Assertion';
var expect = 'No Assertion';

try
{
  eval('getter\n');
}
catch(e)
{
}

assert.sameValue(expect, actual, summary);
