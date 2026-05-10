/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Object.prototype.toLocaleString
info: bugzilla.mozilla.org/show_bug.cgi?id=653789
esid: pending
---*/

var toLocaleString = Object.prototype.toLocaleString;

/*
 * 1. Let O be the result of calling ToObject passing the this value as the
 *    argument.
 */
assert.throws(TypeError, function() { toLocaleString.call(null); });
assert.throws(TypeError, function() { toLocaleString.call(undefined); });
assert.throws(TypeError, function() { toLocaleString.apply(null); });
assert.throws(TypeError, function() { toLocaleString.apply(undefined); });


/*
 * 2. Let toString be the result of calling the [[Get]] internal method of O
 *    passing "toString" as the argument.
 */
try
{
  toLocaleString.call({ get toString() { throw 17; } });
  throw new Error("didn't throw");
}
catch (e)
{
  assert.sameValue(e, 17);
}


/* 3. If IsCallable(toString) is false, throw a TypeError exception. */
assert.throws(TypeError, function() { toLocaleString.call({ toString: 12 }); });
assert.throws(TypeError, function() { toLocaleString.call({ toString: 0.3423423452352e9 }); });
assert.throws(TypeError, function() { toLocaleString.call({ toString: undefined }); });
assert.throws(TypeError, function() { toLocaleString.call({ toString: false }); });
assert.throws(TypeError, function() { toLocaleString.call({ toString: [] }); });
assert.throws(TypeError, function() { toLocaleString.call({ toString: {} }); });
assert.throws(TypeError, function() { toLocaleString.call({ toString: new String }); });
assert.throws(TypeError, function() { toLocaleString.call({ toString: new Number(7.7) }); });
assert.throws(TypeError, function() { toLocaleString.call({ toString: new Boolean(true) }); });
assert.throws(TypeError, function() { toLocaleString.call({ toString: JSON }); });

assert.throws(TypeError, function() { toLocaleString.call({ valueOf: 0, toString: 12 }); });
assert.throws(TypeError, function() { toLocaleString.call({ valueOf: 0, toString: 0.3423423452352e9 }); });
assert.throws(TypeError, function() { toLocaleString.call({ valueOf: 0, toString: undefined }); });
assert.throws(TypeError, function() { toLocaleString.call({ valueOf: 0, toString: false }); });
assert.throws(TypeError, function() { toLocaleString.call({ valueOf: 0, toString: [] }); });
assert.throws(TypeError, function() { toLocaleString.call({ valueOf: 0, toString: {} }); });
assert.throws(TypeError, function() { toLocaleString.call({ valueOf: 0, toString: new String }); });
assert.throws(TypeError, function() { toLocaleString.call({ valueOf: 0, toString: new Number(7.7) }); });
assert.throws(TypeError, function() { toLocaleString.call({ valueOf: 0, toString: new Boolean(true) }); });
assert.throws(TypeError, function() { toLocaleString.call({ valueOf: 0, toString: JSON }); });


/*
 * 4. Return the result of calling the [[Call]] internal method of toString
 *    passing O as the this value and no arguments.
 */
assert.sameValue(toLocaleString.call({ get toString() { return function() { return "foo"; } } }),
         "foo");

var obj = { toString: function() { assert.sameValue(this, obj); assert.sameValue(arguments.length, 0); return 5; } };
assert.sameValue(toLocaleString.call(obj), 5);

assert.sameValue(toLocaleString.call({ toString: function() { return obj; } }), obj);

assert.sameValue(toLocaleString.call({ toString: function() { return obj; },
                               valueOf: function() { return "abc"; } }),
         obj);
