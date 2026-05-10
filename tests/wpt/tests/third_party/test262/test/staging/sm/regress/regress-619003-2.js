/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
var expect = "global";
var actual = expect;
function f([actual]) { }
f(["local"]);
assert.sameValue(expect, actual, "ok");
