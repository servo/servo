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
var BUGNUMBER = 531682;
var summary = 'Checking proper wrapping of scope in  eval(source, scope)';
var actual;
var expect;

//-----------------------------------------------------------------------------
var x = 0;

test();
//-----------------------------------------------------------------------------

function scope1() {
    eval('var x = 1;');
    return function() { return x; }
}

function test() {
    // The scope chain in eval should be just scope1() and the global object.
    actual = eval('x', scope1());
    expect = 0;
    assert.sameValue(expect, actual, summary);
}
