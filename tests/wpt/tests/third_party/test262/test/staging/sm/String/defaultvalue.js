/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  [[DefaultValue]] behavior wrong for String with overridden valueOf/toString
info: bugzilla.mozilla.org/show_bug.cgi?id=645464
esid: pending
---*/

// equality

var s = new String("c");
assert.sameValue(s == "c", true);

var s2 = new String();
s2.valueOf = function() { return "foo"; };
assert.sameValue(s2 == "foo", true);

var s3 = new String();
s3.toString = function() { return "bar"; };
assert.sameValue(s3 == "", true);

function testEquality()
{
  var s = new String("c");
  assert.sameValue(s == "c", true);

  var s2 = new String();
  s2.valueOf = function() { return "foo"; };
  assert.sameValue(s2 == "foo", true);

  var s3 = new String();
  s3.toString = function() { return "bar"; };
  assert.sameValue(s3 == "", true);
}
testEquality();


// addition of String to string

var s = new String();
assert.sameValue(s + "", "");

var s2 = new String();
s2.toString = function() { return "bar"; };
assert.sameValue(s2 + "", "");

var s3 = new String();
s3.valueOf = function() { return "baz"; };
assert.sameValue(s3 + "", "baz");

function testStringAddition()
{
  var s = new String();
  assert.sameValue(s + "", "");

  var s2 = new String();
  s2.toString = function() { return "bar"; };
  assert.sameValue(s2 + "", "");

  var s3 = new String();
  s3.valueOf = function() { return "baz"; };
  assert.sameValue(s3 + "", "baz");
}
testStringAddition();


// addition of String to String

var s = new String();
assert.sameValue(s + s, "");

var s2 = new String();
s2.toString = function() { return "baz"; };
assert.sameValue(s2 + s2, "");

var s3 = new String();
s3.valueOf = function() { return "quux"; };
assert.sameValue(s3 + s3, "quuxquux");

function testNonStringAddition()
{
  var s = new String();
  assert.sameValue(s + s, "");

  var s2 = new String();
  s2.toString = function() { return "baz"; };
  assert.sameValue(s2 + s2, "");

  var s3 = new String();
  s3.valueOf = function() { return "quux"; };
  assert.sameValue(s3 + s3, "quuxquux");
}
testNonStringAddition();


// String as bracketed property name

var obj = { primitive: 17, valueOf: 42, toString: 8675309 };

var s = new String("primitive");
assert.sameValue(obj[s], 17);

var s2 = new String("primitive");
s2.valueOf = function() { return "valueOf"; }
assert.sameValue(obj[s2], 17);

var s3 = new String("primitive");
s3.toString = function() { return "toString"; };
assert.sameValue(obj[s3], 8675309);

function testPropertyNameToString()
{
  var obj = { primitive: 17, valueOf: 42, toString: 8675309 };

  var s = new String("primitive");
  assert.sameValue(obj[s], 17);

  var s2 = new String("primitive");
  s2.valueOf = function() { return "valueOf"; }
  assert.sameValue(obj[s2], 17);

  var s3 = new String("primitive");
  s3.toString = function() { return "toString"; };
  assert.sameValue(obj[s3], 8675309);
}
testPropertyNameToString();


// String as property name with |in| operator

var s = new String();
assert.sameValue(s in { "": 5 }, true);

var s2 = new String();
s.toString = function() { return "baz"; };
assert.sameValue(s in { baz: 42 }, true);

var s3 = new String();
s3.valueOf = function() { return "quux"; };
assert.sameValue(s3 in { "": 17 }, true);

function testInOperatorName()
{
  var s = new String();
  assert.sameValue(s in { "": 5 }, true);

  var s2 = new String();
  s.toString = function() { return "baz"; };
  assert.sameValue(s in { baz: 42 }, true);

  var s3 = new String();
  s3.valueOf = function() { return "quux"; };
  assert.sameValue(s3 in { "": 17 }, true);
}
testInOperatorName();
