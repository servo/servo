/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - noStrict
description: |
  Assignments to a property that has a getter but not a setter should not throw a TypeError per ES5 (at least not until strict mode is supported)
info: bugzilla.mozilla.org/show_bug.cgi?id=523846
esid: pending
---*/

var o = { get p() { return "a"; } };

function test1()
{
  o.p = "b";
  assert.sameValue(o.p, "a");
}

function test2()
{
  function T() {}
  T.prototype = o;
  y = new T();
  y.p = "b";
  assert.sameValue(y.p, "a");
}

function strictTest1()
{
  "use strict";

  o.p = "b"; // strict-mode violation here
  assert.sameValue(o.p, "a");
}

function strictTest2()
{
  "use strict";

  function T() {}
  T.prototype = o;
  y = new T;
  y.p = "b";  // strict-mode violation here
  assert.sameValue(y.p, "a");
}

test1();
test2();
assert.throws(TypeError, strictTest1);
assert.throws(TypeError, strictTest2);
