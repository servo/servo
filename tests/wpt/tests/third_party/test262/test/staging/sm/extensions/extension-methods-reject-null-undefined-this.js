/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  ECMAScript built-in methods that immediately throw when |this| is |undefined| or |null| (due to CheckObjectCoercible, ToObject, or ToString)
info: bugzilla.mozilla.org/show_bug.cgi?id=619283
esid: pending
---*/

// This test fills out for the non-standard methods which
// non262/misc/builtin-methods-reject-null-undefined-this.js declines to test.

var ClassToMethodMap =
  {
    Object:   ["toSource"],
    Function: ["toSource"],
    Array:    ["toSource"],
    String:   ["toSource"],
    Boolean:  ["toSource"],
    Number:   ["toSource"],
    Date:     ["toSource"],
    RegExp:   ["toSource"],
    Error:    ["toSource"],
  };

var badThisValues = [null, undefined];

function testMethod(Class, className, method)
{
  var expr;

  // Try out explicit this values
  for (var i = 0, sz = badThisValues.length; i < sz; i++)
  {
    var badThis = badThisValues[i];

    expr = className + ".prototype." + method + ".call(" + badThis + ")";
    assert.throws(TypeError, function() {
      Class.prototype[method].call(badThis);
    }, "wrong error for " + expr);

    expr = className + ".prototype." + method + ".apply(" + badThis + ")";
    assert.throws(TypeError, function() {
      Class.prototype[method].apply(badThis);
    }, "wrong error for " + expr);
  }

  // ..and for good measure..

  expr = "(0, " + className + ".prototype." + method + ")()"
  assert.throws(TypeError, function() {
    // comma operator to call GetValue() on the method and de-Reference it
    (0, Class.prototype[method])();
  }, "wrong error for " + expr);
}

for (var className in ClassToMethodMap)
{
  var Class = this[className];

  var methodNames = ClassToMethodMap[className];
  for (var i = 0, sz = methodNames.length; i < sz; i++)
  {
    var method = methodNames[i];
    testMethod(Class, className, method);
  }
}
