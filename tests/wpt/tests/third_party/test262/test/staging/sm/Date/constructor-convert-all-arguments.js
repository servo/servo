/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  new Date(...) must convert *all* arguments to number, not return NaN early if a non-finite argument is encountered
info: bugzilla.mozilla.org/show_bug.cgi?id=1160356
esid: pending
---*/

function expectThrowTypeError(f, i)
{
  try
  {
    f();
    throw new Error("didn't throw");
  }
  catch (e)
  {
    assert.sameValue(e, 42, "index " + i + ": expected 42, got " + e);
  }
}

var bad =
  { toString: function() { throw 17; }, valueOf: function() { throw 42; } };

var funcs =
 [
  function() { new Date(bad); },

  function() { new Date(NaN, bad); },
  function() { new Date(Infinity, bad); },
  function() { new Date(1970, bad); },

  function() { new Date(1970, NaN, bad); },
  function() { new Date(1970, Infinity, bad); },
  function() { new Date(1970, 4, bad); },

  function() { new Date(1970, 4, NaN, bad); },
  function() { new Date(1970, 4, Infinity, bad); },
  function() { new Date(1970, 4, 17, bad); },

  function() { new Date(1970, 4, 17, NaN, bad); },
  function() { new Date(1970, 4, 17, Infinity, bad); },
  function() { new Date(1970, 4, 17, 13, bad); },

  function() { new Date(1970, 4, 17, 13, NaN, bad); },
  function() { new Date(1970, 4, 17, 13, Infinity, bad); },
  function() { new Date(1970, 4, 17, 13, 37, bad); },

  function() { new Date(1970, 4, 17, 13, 37, NaN, bad); },
  function() { new Date(1970, 4, 17, 13, 37, Infinity, bad); },
  function() { new Date(1970, 4, 17, 13, 37, 23, bad); },
 ];

for (var i = 0, len = funcs.length; i < len; i++)
  expectThrowTypeError(funcs[i]);
