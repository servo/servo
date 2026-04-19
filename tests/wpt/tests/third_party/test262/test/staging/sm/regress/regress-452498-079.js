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
var BUGNUMBER = 452498;
var summary = 'TM: upvar2 regression tests';
var actual = '';
var expect = '';


//-----------------------------------------------------------------------------
test();
//-----------------------------------------------------------------------------

function test()
{
// ------- Comment #79 From Jason Orendorff

  x; var x; function x() { return 0; }

// Assertion failure: !(pn->pn_dflags & flag), at ../jsparse.h:635

  assert.sameValue(expect, actual, summary);
}
