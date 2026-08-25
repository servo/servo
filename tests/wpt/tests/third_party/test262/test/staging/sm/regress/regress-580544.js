/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
var gTestfile = 'regress-580544.js';
//-----------------------------------------------------------------------------
var BUGNUMBER = 580544;
var summary = 'Do not assert: new (this.prototype = this)';
var actual = 'No Crash';
var expect = 'No Crash';


//-----------------------------------------------------------------------------
test();
//-----------------------------------------------------------------------------

function test()
{
  try {
    new (this.prototype = this);
  } catch (e) {
  }

  assert.sameValue(expect, actual, summary);
}
