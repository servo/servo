/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Object.prototype.propertyIsEnumerable
info: bugzilla.mozilla.org/show_bug.cgi?id=619283
esid: pending
---*/

function withToString(fun)
{
  return { toString: fun };
}

function withValueOf(fun)
{
  return { toString: null, valueOf: fun };
}

var propertyIsEnumerable = Object.prototype.propertyIsEnumerable;

/*
 * 1. Let P be ToString(V).
 */
assert.throws(ReferenceError, function()
{
  propertyIsEnumerable(withToString(function() { fahslkjdfhlkjdsl; }));
});
assert.throws(ReferenceError, function()
{
  propertyIsEnumerable.call(null, withToString(function() { fahslkjdfhlkjdsl; }));
});
assert.throws(ReferenceError, function()
{
  propertyIsEnumerable.call(undefined, withToString(function() { fahslkjdfhlkjdsl; }));
});

assert.throws(ReferenceError, function()
{
  propertyIsEnumerable(withValueOf(function() { fahslkjdfhlkjdsl; }));
});
assert.throws(ReferenceError, function()
{
  propertyIsEnumerable.call(null, withValueOf(function() { fahslkjdfhlkjdsl; }));
});
assert.throws(ReferenceError, function()
{
  propertyIsEnumerable.call(undefined, withValueOf(function() { fahslkjdfhlkjdsl; }));
});

assert.throws(SyntaxError, function()
{
  propertyIsEnumerable(withToString(function() { eval("}"); }));
});
assert.throws(SyntaxError, function()
{
  propertyIsEnumerable.call(null, withToString(function() { eval("}"); }));
});
assert.throws(SyntaxError, function()
{
  propertyIsEnumerable.call(undefined, withToString(function() { eval("}"); }));
});

assert.throws(SyntaxError, function()
{
  propertyIsEnumerable(withValueOf(function() { eval("}"); }));
});
assert.throws(SyntaxError, function()
{
  propertyIsEnumerable.call(null, withValueOf(function() { eval("}"); }));
});
assert.throws(SyntaxError, function()
{
  propertyIsEnumerable.call(undefined, withValueOf(function() { eval("}"); }));
});

assert.throws(RangeError, function()
{
  propertyIsEnumerable(withToString(function() { [].length = -1; }));
});
assert.throws(RangeError, function()
{
  propertyIsEnumerable.call(null, withToString(function() { [].length = -1; }));
});
assert.throws(RangeError, function()
{
  propertyIsEnumerable.call(undefined, withToString(function() { [].length = -1; }));
});

assert.throws(RangeError, function()
{
  propertyIsEnumerable(withValueOf(function() { [].length = -1; }));
});
assert.throws(RangeError, function()
{
  propertyIsEnumerable.call(null, withValueOf(function() { [].length = -1; }));
});
assert.throws(RangeError, function()
{
  propertyIsEnumerable.call(undefined, withValueOf(function() { [].length = -1; }));
});

assert.throws(RangeError, function()
{
  propertyIsEnumerable(withToString(function() { [].length = 0.7; }));
});
assert.throws(RangeError, function()
{
  propertyIsEnumerable.call(null, withToString(function() { [].length = 0.7; }));
});
assert.throws(RangeError, function()
{
  propertyIsEnumerable.call(undefined, withToString(function() { [].length = 0.7; }));
});

assert.throws(RangeError, function()
{
  propertyIsEnumerable(withValueOf(function() { [].length = 0.7; }));
});
assert.throws(RangeError, function()
{
  propertyIsEnumerable.call(null, withValueOf(function() { [].length = 0.7; }));
});
assert.throws(RangeError, function()
{
  propertyIsEnumerable.call(undefined, withValueOf(function() { [].length = 0.7; }));
});

/*
 * 2. Let O be the result of calling ToObject passing the this value as the
 *    argument.
 */
assert.throws(TypeError, function() { propertyIsEnumerable("s"); });
assert.throws(TypeError, function() { propertyIsEnumerable.call(null, "s"); });
assert.throws(TypeError, function() { propertyIsEnumerable.call(undefined, "s"); });
assert.throws(TypeError, function() { propertyIsEnumerable(true); });
assert.throws(TypeError, function() { propertyIsEnumerable.call(null, true); });
assert.throws(TypeError, function() { propertyIsEnumerable.call(undefined, true); });
assert.throws(TypeError, function() { propertyIsEnumerable(NaN); });
assert.throws(TypeError, function() { propertyIsEnumerable.call(null, NaN); });
assert.throws(TypeError, function() { propertyIsEnumerable.call(undefined, NaN); });

assert.throws(TypeError, function() { propertyIsEnumerable({}); });
assert.throws(TypeError, function() { propertyIsEnumerable.call(null, {}); });
assert.throws(TypeError, function() { propertyIsEnumerable.call(undefined, {}); });

/*
 * 3. Let desc be the result of calling the [[GetOwnProperty]] internal method
 *    of O passing P as the argument.
 * 4. If desc is undefined, return false.
 */
assert.sameValue(propertyIsEnumerable.call({}, "valueOf"), false);
assert.sameValue(propertyIsEnumerable.call({}, "toString"), false);
assert.sameValue(propertyIsEnumerable.call("s", 1), false);
assert.sameValue(propertyIsEnumerable.call({}, "dsfiodjfs"), false);
assert.sameValue(propertyIsEnumerable.call(true, "toString"), false);
assert.sameValue(propertyIsEnumerable.call({}, "__proto__"), false);

assert.sameValue(propertyIsEnumerable.call(Object, "getOwnPropertyDescriptor"), false);
assert.sameValue(propertyIsEnumerable.call(this, "withToString"), true);
assert.sameValue(propertyIsEnumerable.call("s", "length"), false);
assert.sameValue(propertyIsEnumerable.call("s", 0), true);
assert.sameValue(propertyIsEnumerable.call(Number, "MAX_VALUE"), false);
assert.sameValue(propertyIsEnumerable.call({ x: 9 }, "x"), true);
assert.sameValue(propertyIsEnumerable.call(function() { }, "prototype"), false);
assert.sameValue(propertyIsEnumerable.call(function() { }, "length"), false);
assert.sameValue(propertyIsEnumerable.call(function() { "use strict"; }, "caller"), false);
