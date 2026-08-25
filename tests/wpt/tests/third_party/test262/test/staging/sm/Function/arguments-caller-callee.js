/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - noStrict
description: |
  arguments.caller and arguments.callee are poison pills in ES5, later changed in ES2017 to only poison pill arguments.callee.
info: bugzilla.mozilla.org/show_bug.cgi?id=514563
esid: pending
---*/

// behavior

function bar() { "use strict"; return arguments; }
assert.sameValue(bar().caller, undefined); // No error when accessing arguments.caller in ES2017+
assert.throws(TypeError, function barCallee() { bar().callee; });

function baz() { return arguments; }
assert.sameValue(baz().callee, baz);


// accessor identity

function strictMode() { "use strict"; return arguments; }
var canonicalTTE = Object.getOwnPropertyDescriptor(strictMode(), "callee").get;

var args = strictMode();

var argsCaller = Object.getOwnPropertyDescriptor(args, "caller");
assert.sameValue(argsCaller, undefined);

var argsCallee = Object.getOwnPropertyDescriptor(args, "callee");
assert.sameValue("get" in argsCallee, true);
assert.sameValue("set" in argsCallee, true);
assert.sameValue(argsCallee.get, canonicalTTE);
assert.sameValue(argsCallee.set, canonicalTTE);
