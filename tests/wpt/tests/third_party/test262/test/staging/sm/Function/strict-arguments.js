/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [compareArray.js]
flags:
  - noStrict
description: |
  ES5 strict mode: arguments objects of strict mode functions must copy argument values
info: bugzilla.mozilla.org/show_bug.cgi?id=516255
esid: pending
---*/

/************************
 * NON-STRICT ARGUMENTS *
 ************************/

var obj = {};

function noargs() { return arguments; }

assert.compareArray(noargs(), []);
assert.compareArray(noargs(1), [1]);
assert.compareArray(noargs(2, obj, 8), [2, obj, 8]);

function args(a) { return arguments; }

assert.compareArray(args(), []);
assert.compareArray(args(1), [1]);
assert.compareArray(args(1, obj), [1, obj]);
assert.compareArray(args("foopy"), ["foopy"]);

function assign(a)
{
  a = 17;
  return arguments;
}

assert.compareArray(assign(1), [17]);

function getLaterAssign(a)
{
  var o = arguments;
  a = 17;
  return o;
}

assert.compareArray(getLaterAssign(1), [17]);

function assignElementGetParameter(a)
{
  arguments[0] = 17;
  return a;
}

assert.sameValue(assignElementGetParameter(42), 17);

function assignParameterGetElement(a)
{
  a = 17;
  return arguments[0];
}

assert.sameValue(assignParameterGetElement(42), 17);

function assignArgSub(x, y)
{
  arguments[0] = 3;
  return arguments[0];
}

assert.sameValue(assignArgSub(1), 3);

function assignArgSubParamUse(x, y)
{
  arguments[0] = 3;
  assert.sameValue(x, 3);
  return arguments[0];
}

assert.sameValue(assignArgSubParamUse(1), 3);

function assignArgumentsElement(x, y)
{
  arguments[0] = 3;
  return arguments[Math.random() ? "0" : 0]; // nix arguments[const] optimizations
}

assert.sameValue(assignArgumentsElement(1), 3);

function assignArgumentsElementParamUse(x, y)
{
  arguments[0] = 3;
  assert.sameValue(x, 3);
  return arguments[Math.random() ? "0" : 0]; // nix arguments[const] optimizations
}

assert.sameValue(assignArgumentsElementParamUse(1), 3);

/********************
 * STRICT ARGUMENTS *
 ********************/

function strictNoargs()
{
  "use strict";
  return arguments;
}

assert.compareArray(strictNoargs(), []);
assert.compareArray(strictNoargs(1), [1]);
assert.compareArray(strictNoargs(1, obj), [1, obj]);

function strictArgs(a)
{
  "use strict";
  return arguments;
}

assert.compareArray(strictArgs(), []);
assert.compareArray(strictArgs(1), [1]);
assert.compareArray(strictArgs(1, obj), [1, obj]);

function strictAssign(a)
{
  "use strict";
  a = 17;
  return arguments;
}

assert.compareArray(strictAssign(), []);
assert.compareArray(strictAssign(1), [1]);
assert.compareArray(strictAssign(1, obj), [1, obj]);

var upper;
function strictAssignAfter(a)
{
  "use strict";
  upper = arguments;
  a = 42;
  return upper;
}

assert.compareArray(strictAssignAfter(), []);
assert.compareArray(strictAssignAfter(17), [17]);
assert.compareArray(strictAssignAfter(obj), [obj]);

function strictMaybeAssignOuterParam(p)
{
  "use strict";
  function inner() { p = 17; }
  return arguments;
}

assert.compareArray(strictMaybeAssignOuterParam(), []);
assert.compareArray(strictMaybeAssignOuterParam(42), [42]);
assert.compareArray(strictMaybeAssignOuterParam(obj), [obj]);

function strictAssignOuterParam(p)
{
  "use strict";
  function inner() { p = 17; }
  inner();
  return arguments;
}

assert.compareArray(strictAssignOuterParam(), []);
assert.compareArray(strictAssignOuterParam(17), [17]);
assert.compareArray(strictAssignOuterParam(obj), [obj]);

function strictAssignOuterParamPSYCH(p)
{
  "use strict";
  function inner(p) { p = 17; }
  inner();
  return arguments;
}

assert.compareArray(strictAssignOuterParamPSYCH(), []);
assert.compareArray(strictAssignOuterParamPSYCH(17), [17]);
assert.compareArray(strictAssignOuterParamPSYCH(obj), [obj]);

function strictEval(code, p)
{
  "use strict";
  eval(code);
  return arguments;
}

assert.compareArray(strictEval("1", 2), ["1", 2]);
assert.compareArray(strictEval("arguments"), ["arguments"]);
assert.compareArray(strictEval("p = 2"), ["p = 2"]);
assert.compareArray(strictEval("p = 2", 17), ["p = 2", 17]);
assert.compareArray(strictEval("arguments[0] = 17"), [17]);
assert.compareArray(strictEval("arguments[0] = 17", 42), [17, 42]);

function strictMaybeNestedEval(code, p)
{
  "use strict";
  function inner() { eval(code); }
  return arguments;
}

assert.compareArray(strictMaybeNestedEval("1", 2), ["1", 2]);
assert.compareArray(strictMaybeNestedEval("arguments"), ["arguments"]);
assert.compareArray(strictMaybeNestedEval("p = 2"), ["p = 2"]);
assert.compareArray(strictMaybeNestedEval("p = 2", 17), ["p = 2", 17]);

function strictNestedEval(code, p)
{
  "use strict";
  function inner() { eval(code); }
  inner();
  return arguments;
}

assert.compareArray(strictNestedEval("1", 2), ["1", 2]);
assert.compareArray(strictNestedEval("arguments"), ["arguments"]);
assert.compareArray(strictNestedEval("p = 2"), ["p = 2"]);
assert.compareArray(strictNestedEval("p = 2", 17), ["p = 2", 17]);
assert.compareArray(strictNestedEval("arguments[0] = 17"), ["arguments[0] = 17"]);
assert.compareArray(strictNestedEval("arguments[0] = 17", 42), ["arguments[0] = 17", 42]);

function strictAssignArguments(a)
{
  "use strict";
  arguments[0] = 42;
  return a;
}

assert.sameValue(strictAssignArguments(), undefined);
assert.sameValue(strictAssignArguments(obj), obj);
assert.sameValue(strictAssignArguments(17), 17);

function strictAssignParameterGetElement(a)
{
  "use strict";
  a = 17;
  return arguments[0];
}

assert.sameValue(strictAssignParameterGetElement(42), 42);

function strictAssignArgSub(x, y)
{
  "use strict";
  arguments[0] = 3;
  return arguments[0];
}

assert.sameValue(strictAssignArgSub(1), 3);

function strictAssignArgSubParamUse(x, y)
{
  "use strict";
  arguments[0] = 3;
  assert.sameValue(x, 1);
  return arguments[0];
}

assert.sameValue(strictAssignArgSubParamUse(1), 3);

function strictAssignArgumentsElement(x, y)
{
  "use strict";
  arguments[0] = 3;
  return arguments[Math.random() ? "0" : 0]; // nix arguments[const] optimizations
}

assert.sameValue(strictAssignArgumentsElement(1), 3);

function strictAssignArgumentsElementParamUse(x, y)
{
  "use strict";
  arguments[0] = 3;
  assert.sameValue(x, 1);
  return arguments[Math.random() ? "0" : 0]; // nix arguments[const] optimizations
}

assert.sameValue(strictAssignArgumentsElementParamUse(1), 3);

function strictNestedAssignShadowVar(p)
{
  "use strict";
  function inner()
  {
    var p = 12;
    function innermost() { p = 1776; return 12; }
    return innermost();
  }
  return arguments;
}

assert.compareArray(strictNestedAssignShadowVar(), []);
assert.compareArray(strictNestedAssignShadowVar(99), [99]);
assert.compareArray(strictNestedAssignShadowVar(""), [""]);
assert.compareArray(strictNestedAssignShadowVar(obj), [obj]);

function strictNestedAssignShadowCatch(p)
{
  "use strict";
  function inner()
  {
    try
    {
    }
    catch (p)
    {
      var f = function innermost() { p = 1776; return 12; };
      f();
    }
  }
  return arguments;
}

assert.compareArray(strictNestedAssignShadowCatch(), []);
assert.compareArray(strictNestedAssignShadowCatch(99), [99]);
assert.compareArray(strictNestedAssignShadowCatch(""), [""]);
assert.compareArray(strictNestedAssignShadowCatch(obj), [obj]);

function strictNestedAssignShadowCatchCall(p)
{
  "use strict";
  function inner()
  {
    try
    {
    }
    catch (p)
    {
      var f = function innermost() { p = 1776; return 12; };
      f();
    }
  }
  inner();
  return arguments;
}

assert.compareArray(strictNestedAssignShadowCatchCall(), []);
assert.compareArray(strictNestedAssignShadowCatchCall(99), [99]);
assert.compareArray(strictNestedAssignShadowCatchCall(""), [""]);
assert.compareArray(strictNestedAssignShadowCatchCall(obj), [obj]);

function strictNestedAssignShadowFunction(p)
{
  "use strict";
  function inner()
  {
    function p() { }
    p = 1776;
  }
  return arguments;
}

assert.compareArray(strictNestedAssignShadowFunction(), []);
assert.compareArray(strictNestedAssignShadowFunction(99), [99]);
assert.compareArray(strictNestedAssignShadowFunction(""), [""]);
assert.compareArray(strictNestedAssignShadowFunction(obj), [obj]);

function strictNestedAssignShadowFunctionCall(p)
{
  "use strict";
  function inner()
  {
    function p() { }
    p = 1776;
  }
  return arguments;
}

assert.compareArray(strictNestedAssignShadowFunctionCall(), []);
assert.compareArray(strictNestedAssignShadowFunctionCall(99), [99]);
assert.compareArray(strictNestedAssignShadowFunctionCall(""), [""]);
assert.compareArray(strictNestedAssignShadowFunctionCall(obj), [obj]);

function strictNestedShadowAndMaybeEval(code, p)
{
  "use strict";
  function inner(p) { eval(code); }
  return arguments;
}

assert.compareArray(strictNestedShadowAndMaybeEval("1", 2), ["1", 2]);
assert.compareArray(strictNestedShadowAndMaybeEval("arguments"), ["arguments"]);
assert.compareArray(strictNestedShadowAndMaybeEval("p = 2"), ["p = 2"]);
assert.compareArray(strictNestedShadowAndMaybeEval("p = 2", 17), ["p = 2", 17]);
assert.compareArray(strictNestedShadowAndMaybeEval("arguments[0] = 17"), ["arguments[0] = 17"]);
assert.compareArray(strictNestedShadowAndMaybeEval("arguments[0] = 17", 42), ["arguments[0] = 17", 42]);

function strictNestedShadowAndEval(code, p)
{
  "use strict";
  function inner(p) { eval(code); }
  return arguments;
}

assert.compareArray(strictNestedShadowAndEval("1", 2), ["1", 2]);
assert.compareArray(strictNestedShadowAndEval("arguments"), ["arguments"]);
assert.compareArray(strictNestedShadowAndEval("p = 2"), ["p = 2"]);
assert.compareArray(strictNestedShadowAndEval("p = 2", 17), ["p = 2", 17]);
assert.compareArray(strictNestedShadowAndEval("arguments[0] = 17"), ["arguments[0] = 17"]);
assert.compareArray(strictNestedShadowAndEval("arguments[0] = 17", 42), ["arguments[0] = 17", 42]);

function strictEvalContainsMutation(code)
{
  "use strict";
  return eval(code);
}

assert.compareArray(strictEvalContainsMutation("code = 17; arguments"), ["code = 17; arguments"]);
assert.compareArray(strictEvalContainsMutation("arguments[0] = 17; arguments"), [17]);
assert.sameValue(strictEvalContainsMutation("arguments[0] = 17; code"), "arguments[0] = 17; code");

function strictNestedAssignShadowFunctionName(p)
{
  "use strict";
  function inner()
  {
    function p() { p = 1776; }
    p();
  }
  inner();
  return arguments;
}

assert.compareArray(strictNestedAssignShadowFunctionName(), []);
assert.compareArray(strictNestedAssignShadowFunctionName(99), [99]);
assert.compareArray(strictNestedAssignShadowFunctionName(""), [""]);
assert.compareArray(strictNestedAssignShadowFunctionName(obj), [obj]);
