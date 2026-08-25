/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Various quirks of setting array length properties to objects
info: bugzilla.mozilla.org/show_bug.cgi?id=657298
esid: pending
---*/

function invokeConversionTwice1()
{
  var count = 0;
  [].length = { valueOf: function() { count++; return 1; } };
  assert.sameValue(count, 2);
}
invokeConversionTwice1();

function invokeConversionTwice2()
{
  var count = 0;
  [].length = { toString: function() { count++; return 1; }, valueOf: null };
  assert.sameValue(count, 2);
}
invokeConversionTwice2();

function dontOverwriteError1()
{
  assert.throws(TypeError, function() {
    [].length = { valueOf: {}, toString: {} };
  }, "expected a TypeError running out of conversion options");
}
dontOverwriteError1();

function dontOverwriteError2()
{
  try
  {
    [].length = { valueOf: function() { throw "error"; } };
    throw new Error("didn't throw a TypeError");
  }
  catch (e)
  {
    assert.sameValue(e, "error", "expected 'error' from failed conversion, got " + e);
  }
}
dontOverwriteError2();
