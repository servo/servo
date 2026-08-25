/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Don't assert parsing a for-in/of loop whose target is a name, where the expression being iterated over contains a string containing an index
info: bugzilla.mozilla.org/show_bug.cgi?id=1235640
esid: pending
---*/

function f()
{
  var x;
  for (x in "9")
    continue;
  assert.sameValue(x, "0");
}

f();

function g()
{
  "use strict";
  var x = "unset";
  for (x in arguments)
    continue;
  assert.sameValue(x, "unset");
}

g();
