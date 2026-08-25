/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/

var f;
try
{
  f = eval("(function literalInside() { return { set 'c d e'(v) { } }; })");
}
catch (e)
{
  assert.sameValue(true, false,
           "string-literal property name in setter in object literal in " +
           "function statement failed to parse: " + e);
}

var fstr = "" + f;
assert.sameValue(fstr.indexOf("set") < fstr.indexOf("c d e"), true,
         "should be using new-style syntax with string literal in place of " +
         "property identifier");
assert.sameValue(fstr.indexOf("setter") < 0, true, "using old-style syntax?");

var o = f();
assert.sameValue("c d e" in o, true, "missing the property?");
assert.sameValue("set" in Object.getOwnPropertyDescriptor(o, "c d e"), true,
         "'c d e' property not a setter?");

var ostr = Object.getOwnPropertyDescriptor(o, "c d e").set + o;
assert.sameValue(ostr.indexOf("set") < ostr.indexOf("c d e"), true,
        "should be using new-style syntax when getting the source of a " +
        "getter/setter while decompiling an object" + ostr);
assert.sameValue(ostr.indexOf("setter") < 0, true, "using old-style syntax?");
