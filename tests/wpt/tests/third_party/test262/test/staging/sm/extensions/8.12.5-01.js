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
var BUGNUMBER = 523846;
var summary =
  "Assignments to a property that has a getter but not a setter should not " +
  "throw a TypeError per ES5 (at least not until strict mode is supported)";
var actual = "Early failure";
var expect = "No errors";


var o = { get p() { return "a"; } };

function test1()
{
  o.p = "b"; // strict-mode violation here
  assert.sameValue(o.p, "a");
}

function test2()
{
  function T() {}
  T.prototype = o;
  y = new T();
  y.p = "b";  // strict-mode violation here
  assert.sameValue(y.p, "a");
}


var errors = [];
try
{
  try
  {
    test1();
  }
  catch (e)
  {
    errors.push(e);
  }

  try
  {
    test2();
  }
  catch (e)
  {
    errors.push(e);
  }
}
catch (e)
{
  errors.push("Unexpected error: " + e);
}
finally
{
  actual = errors.length > 0 ? errors.join(", ") : "No errors";
}

assert.sameValue(expect, actual, summary);
