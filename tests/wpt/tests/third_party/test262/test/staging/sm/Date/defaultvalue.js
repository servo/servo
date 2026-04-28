/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  [[DefaultValue]] behavior wrong for Date with overridden valueOf/toString
info: bugzilla.mozilla.org/show_bug.cgi?id=645464
esid: pending
---*/

function allTests(Date)
{
  var DS = new Date(2010, 1, 1).toString();

  // equality

  var d = new Date(2010, 1, 1);
  assert.sameValue(d == DS, true);

  var d2 = new Date(2010, 1, 1);
  d2.valueOf = function() { assert.sameValue(arguments.length, 0); return 17; };
  assert.sameValue(d2 == DS, true);

  var d3 = new Date(2010, 1, 1);
  d3.toString = function() { return 42; };
  assert.sameValue(d3 == 42, true);

  function testEquality()
  {
    var d = new Date(2010, 1, 1);
    assert.sameValue(d == DS, true);

    var d2 = new Date(2010, 1, 1);
    d2.valueOf = function() { assert.sameValue(arguments.length, 0); return 17; };
    assert.sameValue(d2 == DS, true);

    var d3 = new Date(2010, 1, 1);
    d3.toString = function() { return 42; };
    assert.sameValue(d3 == 42, true);
  }
  testEquality();


  // addition of Date to number

  var d = new Date(2010, 1, 1);
  assert.sameValue(d + 5, DS + "5");

  var d2 = new Date(2010, 1, 1);
  d2.toString = function() { return 9; };
  assert.sameValue(d2 + 3, 9 + 3);

  var d3 = new Date(2010, 1, 1);
  d3.valueOf = function() { assert.sameValue(arguments.length, 0); return 17; };
  assert.sameValue(d3 + 5, DS + "5");

  function testDateNumberAddition()
  {
    var d = new Date(2010, 1, 1);
    assert.sameValue(d + 5, DS + "5");

    var d2 = new Date(2010, 1, 1);
    d2.toString = function() { return 9; };
    assert.sameValue(d2 + 3, 9 + 3);

    var d3 = new Date(2010, 1, 1);
    d3.valueOf = function() { assert.sameValue(arguments.length, 0); return 17; };
    assert.sameValue(d3 + 5, DS + "5");
  }
  testDateNumberAddition();


  // addition of Date to Date

  var d = new Date(2010, 1, 1);
  assert.sameValue(d + d, DS + DS);

  var d2 = new Date(2010, 1, 1);
  d2.toString = function() { return 5; };
  assert.sameValue(d2 + d2, 10);

  var d3 = new Date(2010, 1, 1);
  d3.valueOf = function() { assert.sameValue(arguments.length, 0); return 8.5; };
  assert.sameValue(d3 + d3, DS + DS);

  function testDateDateAddition()
  {
    var d = new Date(2010, 1, 1);
    assert.sameValue(d + d, DS + DS);

    var d2 = new Date(2010, 1, 1);
    d2.toString = function() { return 5; };
    assert.sameValue(d2 + d2, 10);

    var d3 = new Date(2010, 1, 1);
    d3.valueOf = function() { assert.sameValue(arguments.length, 0); return 8.5; };
    assert.sameValue(d3 + d3, DS + DS);
  }
  testDateDateAddition();


  // Date as bracketed property name

  var obj = { 8: 42, 9: 73 };
  obj[DS] = 17;

  var d = new Date(2010, 1, 1);
  assert.sameValue(obj[d], 17);

  var d2 = new Date(2010, 1, 1);
  d2.valueOf = function() { assert.sameValue(arguments.length, 0); return 8; }
  assert.sameValue(obj[d2], 17);

  var d3 = new Date(2010, 1, 1);
  d3.toString = function() { return 9; };
  assert.sameValue(obj[d3], 73);

  function testPropertyName()
  {
    var obj = { 8: 42, 9: 73 };
    obj[DS] = 17;

    var d = new Date(2010, 1, 1);
    assert.sameValue(obj[d], 17);

    var d2 = new Date(2010, 1, 1);
    d2.valueOf = function() { assert.sameValue(arguments.length, 0); return 8; }
    assert.sameValue(obj[d2], 17);

    var d3 = new Date(2010, 1, 1);
    d3.toString = function() { return 9; };
    assert.sameValue(obj[d3], 73);
  }
  testPropertyName();


  // Date as property name with |in| operator

  var obj = {};
  obj[DS] = 5;

  var d = new Date(2010, 1, 1);
  assert.sameValue(d in obj, true);

  var d2 = new Date(2010, 1, 1);
  d2.toString = function() { return "baz"; };
  assert.sameValue(d2 in { baz: 42 }, true);

  var d3 = new Date(2010, 1, 1);
  d3.valueOf = function() { assert.sameValue(arguments.length, 0); return "quux"; };
  assert.sameValue(d3 in obj, true);

  function testInOperatorName()
  {
    var obj = {};
    obj[DS] = 5;

    var d = new Date(2010, 1, 1);
    assert.sameValue(d in obj, true);

    var d2 = new Date(2010, 1, 1);
    d2.toString = function() { return "baz"; };
    assert.sameValue(d2 in { baz: 42 }, true);

    var d3 = new Date(2010, 1, 1);
    d3.valueOf = function() { assert.sameValue(arguments.length, 0); return "quux"; };
    assert.sameValue(d3 in obj, true);
  }
  testInOperatorName();
}

allTests(Date);
allTests($262.createRealm().global.Date);
