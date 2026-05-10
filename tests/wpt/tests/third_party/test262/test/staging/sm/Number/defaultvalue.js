/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  [[DefaultValue]] behavior wrong for Number with overridden valueOf/toString
info: bugzilla.mozilla.org/show_bug.cgi?id=645464
esid: pending
---*/

// equality

var n = new Number();
assert.sameValue(n == 0, true);

var n2 = new Number();
n2.valueOf = function() { return 17; };
assert.sameValue(n2 == 17, true);

var n3 = new Number();
n3.toString = function() { return 42; };
assert.sameValue(n3 == 0, true);

function testEquality()
{
  var n = new Number();
  assert.sameValue(n == 0, true);

  var n2 = new Number();
  n2.valueOf = function() { return 17; };
  assert.sameValue(n2 == 17, true);

  var n3 = new Number();
  n3.toString = function() { return 42; };
  assert.sameValue(n3 == 0, true);
}
testEquality();


// addition of Number to number

var n = new Number();
assert.sameValue(n + 5, 5);

var n2 = new Number();
n2.toString = function() { return 9; };
assert.sameValue(n2 + 3, 3);

var n3 = new Number();
n3.valueOf = function() { return 17; };
assert.sameValue(n3 + 5, 22);

function testNumberAddition()
{
  var n = new Number();
  assert.sameValue(n + 5, 5);

  var n2 = new Number();
  n2.toString = function() { return 9; };
  assert.sameValue(n2 + 3, 3);

  var n3 = new Number();
  n3.valueOf = function() { return 17; };
  assert.sameValue(n3 + 5, 22);
}
testNumberAddition();


// addition of Number to Number

var n = new Number();
assert.sameValue(n + n, 0);

var n2 = new Number();
n2.toString = function() { return 5; };
assert.sameValue(n2 + n2, 0);

var n3 = new Number();
n3.valueOf = function() { return 8.5; };
assert.sameValue(n3 + n3, 17);

function testNonNumberAddition()
{
  var n = new Number();
  assert.sameValue(n + n, 0);

  var n2 = new Number();
  n2.toString = function() { return 5; };
  assert.sameValue(n2 + n2, 0);

  var n3 = new Number();
  n3.valueOf = function() { return 8.5; };
  assert.sameValue(n3 + n3, 17);
}
testNonNumberAddition();


// Number as bracketed property name

var obj = { 0: 17, 8: 42, 9: 8675309 };

var n = new Number();
assert.sameValue(obj[n], 17);

var n2 = new Number();
n2.valueOf = function() { return 8; }
assert.sameValue(obj[n2], 17);

var n3 = new Number();
n3.toString = function() { return 9; };
assert.sameValue(obj[n3], 8675309);

function testPropertyNameToNumber()
{
  var obj = { 0: 17, 8: 42, 9: 8675309 };

  var n = new Number();
  assert.sameValue(obj[n], 17);

  var n2 = new Number();
  n2.valueOf = function() { return 8; }
  assert.sameValue(obj[n2], 17);

  var n3 = new Number();
  n3.toString = function() { return 9; };
  assert.sameValue(obj[n3], 8675309);
}
testPropertyNameToNumber();


// Number as property name with |in| operator

var n = new Number();
assert.sameValue(n in { 0: 5 }, true);

var n2 = new Number();
n2.toString = function() { return "baz"; };
assert.sameValue(n2 in { baz: 42 }, true);

var n3 = new Number();
n3.valueOf = function() { return "quux"; };
assert.sameValue(n3 in { 0: 17 }, true);

function testInOperatorName()
{
  var n = new Number();
  assert.sameValue(n in { 0: 5 }, true);

  var n2 = new Number();
  n2.toString = function() { return "baz"; };
  assert.sameValue(n2 in { baz: 42 }, true);

  var n3 = new Number();
  n3.valueOf = function() { return "quux"; };
  assert.sameValue(n3 in { 0: 17 }, true);
}
testInOperatorName();
