/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
var s = "grape";
function f() { "use strict"; return this; }
var p = new Proxy(f, {});
String.prototype.p = p;
assert.sameValue(s.p(), "grape");

