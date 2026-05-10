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

// We can't just exhaustively loop over everything because 1) method properties
// might be extensions with special |this| handling, and 2) some methods don't
// *quite* immediately throw a TypeError, first thing, if |this| is |undefined|
// or |null|, or their algorithms are very slightly ambiguous about whether they
// do.  Why?  Ipse-dixitism.  *shrug*

var ClassToMethodMap =
  {
    Object:  [/* "toString" has special |this| handling */
              "toLocaleString", "valueOf", "hasOwnProperty",
              /*
               * "isPrototypeOf" has special |this| handling already tested in
               * non262/Object/isPrototypeOf.js.
               */
              /*
               * "isPrototypeOf" has special |this| handling already tested in
               * non262/Object/propertyIsEnumerable.js.
               */
              "__defineGetter__", "__defineSetter__",
              "__lookupGetter__", "__lookupSetter__",
              ],
    // Function methods often don't ToObject(this) as their very first step,
    // and they're already stepwise well-tested such that manual tests here
    // would be redundant.
    Array:   ["toString", "toLocaleString", "concat", "join", "pop", "push",
              "reverse", "shift", "slice", "sort", "splice", "unshift",
              "indexOf", "lastIndexOf", "every", "some", "forEach", "map",
              "filter", "reduce", "reduceRight"],
    String:  ["toString", "valueOf", "charAt", "charCodeAt", "concat",
              "indexOf", "lastIndexOf", "localeCompare", "match", "replace",
              "search", "slice", "split", "substring", "toLowerCase",
              "toLocaleLowerCase", "toUpperCase", "toLocaleUpperCase", "trim",
              "bold", "italics", "fixed", "fontsize",
              "fontcolor", "link", "anchor", "strike", "small", "big", "blink",
              "sup", "sub", "substr", "trimLeft", "trimRight",
              ],
    Boolean: ["toString", "valueOf"],
    Number:  ["toString", "toLocaleString", "valueOf",
              /*
               * toFixed doesn't *immediately* test |this| for number or
               * Number-ness, but because the ToInteger(void 0) which arguably
               * precedes it in the toFixed algorithm won't throw in this test,
               * we don't need to specially test it.
               */
              "toFixed",
              "toExponential", "toPrecision"],
    Date:    ["toDateString", "toTimeString", "toLocaleString",
              "toLocaleDateString", "toLocaleTimeString", "valueOf", "getTime",
              "getFullYear", "getUTCFullYear", "getMonth", "getUTCMonth",
              "getDate", "getUTCDate", "getDay", "getUTCDay", "getHours",
              "getUTCHours", "getMinutes", "getUTCMinutes", "getSeconds",
              "getUTCSeconds", "getMilliseconds", "getUTCMilliseconds",
              /*
               * toFixed doesn't *immediately* test |this| for number or
               * Number-ness, but because the TimeClip(ToNumber(void 0)) which
               * arguably precedes it in the setTime algorithm won't throw in
               * this test, we don't need to specially test it.
               */
              "setTime",
              "getTimezoneOffset", "setMilliseconds", "setUTCMilliseconds",
              "setSeconds", "setUTCSeconds", "setMinutes", "setUTCMinutes",
              "setHours", "setUTCHours", "setDate", "setUTCDate",  "setMonth",
              "setUTCMonth", "setFullYear", "setUTCFullYear", "toUTCString",
              "toISOString", "toJSON",
              "getYear", "setYear",  "toGMTString"],
    RegExp:  ["exec", "test", "toString"],
    Error:   ["toString"],
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

  expr = "(0, " + className + ".prototype." + method + ")()";
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
