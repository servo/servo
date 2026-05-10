/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - noStrict
description: |
  |delete x| inside a function in eval code, where that eval code includes |var x| at top level, actually does delete the binding for x
info: bugzilla.mozilla.org/show_bug.cgi?id=616294
esid: pending
---*/

var f;

function testOuterVar()
{
  return eval("var x; (function() { return delete x; })");
}

f = testOuterVar();

assert.sameValue(f(), true); // configurable, so remove => true
assert.sameValue(f(), true); // not there => true (only non-configurable => false)


function testOuterFunction()
{
  return eval("function x() { } (function() { return delete x; })");
}

f = testOuterFunction();

assert.sameValue(f(), true); // configurable, so remove => true
assert.sameValue(f(), true); // not there => true (only non-configurable => false)


function testOuterForVar()
{
  return eval("for (var x; false; ); (function() { return delete x; })");
}

f = testOuterForVar();

assert.sameValue(f(), true); // configurable, so remove => true
assert.sameValue(f(), true); // not there => true (only non-configurable => false)


function testOuterForInVar()
{
  return eval("for (var x in {}); (function() { return delete x; })");
}

f = testOuterForInVar();

assert.sameValue(f(), true); // configurable, so remove => true
assert.sameValue(f(), true); // not there => true (only non-configurable => false)


function testOuterNestedVar()
{
  return eval("for (var q = 0; q < 5; q++) { var x; } (function() { return delete x; })");
}

f = testOuterNestedVar();

assert.sameValue(f(), true); // configurable, so remove => true
assert.sameValue(f(), true); // not there => true (only non-configurable => false)


function testOuterNestedConditionalVar()
{
  return eval("for (var q = 0; q < 5; q++) { if (false) { var x; } } (function() { return delete x; })");
}

f = testOuterNestedConditionalVar();

assert.sameValue(f(), true); // configurable, so remove => true
assert.sameValue(f(), true); // not there => true (only non-configurable => false)


function testVarInWith()
{
  return eval("with ({}) { var x; } (function() { return delete x; })");
}

f = testVarInWith();

assert.sameValue(f(), true); // configurable, so remove => true
assert.sameValue(f(), true); // not there => true (only non-configurable => false)


function testForVarInWith()
{
  return eval("with ({}) { for (var x = 0; x < 5; x++); } (function() { return delete x; })");
}

f = testForVarInWith();

assert.sameValue(f(), true); // configurable, so remove => true
assert.sameValue(f(), true); // not there => true (only non-configurable => false)


function testForInVarInWith()
{
  return eval("with ({}) { for (var x in {}); } (function() { return delete x; })");
}

f = testForInVarInWith();

assert.sameValue(f(), true); // configurable, so remove => true
assert.sameValue(f(), true); // not there => true (only non-configurable => false)


function testUnknown()
{
  return eval("nameToDelete = 17; (function() { return delete nameToDelete; })");
}

f = testUnknown();

assert.sameValue(f(), true); // configurable global property, so remove => true
assert.sameValue(f(), true); // not there => true (only non-configurable => false)


function testArgumentShadow()
{
  return eval("var x; (function(x) { return delete x; })");
}

f = testArgumentShadow();

assert.sameValue(f(), false); // non-configurable argument => false


function testArgument()
{
  return eval("(function(x) { return delete x; })");
}

f = testArgument();

assert.sameValue(f(), false); // non-configurable argument => false


function testFunctionLocal()
{
  return eval("(function() { var x; return delete x; })");
}

f = testFunctionLocal();

assert.sameValue(f(), false); // defined by function code => not configurable => false
