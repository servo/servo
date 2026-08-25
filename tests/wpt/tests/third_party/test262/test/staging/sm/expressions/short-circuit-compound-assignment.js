// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
const testCasesAnd = [];
const testCasesOr = [];
const testCasesCoalesce = [];


// Assignment to a global variable (JSOp::SetGName).
var globalVar;

function testAnd_GlobalVar(init, value) {
  globalVar = init;
  return [(globalVar &&= value), globalVar];
}
testCasesAnd.push(testAnd_GlobalVar);

function testOr_GlobalVar(init, value) {
  globalVar = init;
  return [(globalVar ||= value), globalVar];
}
testCasesOr.push(testOr_GlobalVar);

function testCoalesce_GlobalVar(init, value) {
  globalVar = init;
  return [(globalVar ??= value), globalVar];
}
testCasesCoalesce.push(testCoalesce_GlobalVar);


// Assignment to a local variable (JSOp::SetLocal).
function testAnd_LocalVar(init, value) {
  let v = init;
  return [(v &&= value), v];
}
testCasesAnd.push(testAnd_LocalVar);

function testOr_LocalVar(init, value) {
  let v = init;
  return [(v ||= value), v];
}
testCasesOr.push(testOr_LocalVar);

function testCoalesce_LocalVar(init, value) {
  let v = init;
  return [(v ??= value), v];
}
testCasesCoalesce.push(testCoalesce_LocalVar);


// Assignment to a parameter (JSOp::SetArg).
function testAnd_Arg(init, value) {
  function f(v) {
    return [(v &&= value), v];
  }
  return f(init);
}
testCasesAnd.push(testAnd_Arg);

function testOr_Arg(init, value) {
  function f(v) {
    return [(v ||= value), v];
  }
  return f(init);
}
testCasesOr.push(testOr_Arg);

function testCoalesce_Arg(init, value) {
  function f(v) {
    return [(v ??= value), v];
  }
  return f(init);
}
testCasesCoalesce.push(testCoalesce_Arg);


// Assignment to a closed over variable (JSOp::SetAliasedVar).
function testAnd_ClosedOver(init, value) {
  let v = init;
  function f() {
    return (v &&= value);
  }
  return [f(), v];
}
testCasesAnd.push(testAnd_ClosedOver);

function testOr_ClosedOver(init, value) {
  let v = init;
  function f() {
    return (v ||= value);
  }
  return [f(), v];
}
testCasesOr.push(testOr_ClosedOver);

function testCoalesce_ClosedOver(init, value) {
  let v = init;
  function f() {
    return (v ??= value);
  }
  return [f(), v];
}
testCasesCoalesce.push(testCoalesce_ClosedOver);


// Assignment to a dynamic name (JSOp::SetName).
function testAnd_DynamicName(init, value) {
  eval("var v = init;");
  return [(v &&= value), v];
}
testCasesAnd.push(testAnd_DynamicName);

function testOr_DynamicName(init, value) {
  eval("var v = init;");
  return [(v ||= value), v];
}
testCasesOr.push(testOr_DynamicName);

function testCoalesce_DynamicName(init, value) {
  eval("var v = init;");
  return [(v ??= value), v];
}
testCasesCoalesce.push(testCoalesce_DynamicName);


// Assignment to a property.
function testAnd_Property(init, value) {
  let obj = {p: init};
  return [(obj.p &&= value), obj.p];
}
testCasesAnd.push(testAnd_Property);

function testOr_Property(init, value) {
  let obj = {p: init};
  return [(obj.p ||= value), obj.p];
}
testCasesOr.push(testOr_Property);

function testCoalesce_Property(init, value) {
  let obj = {p: init};
  return [(obj.p ??= value), obj.p];
}
testCasesCoalesce.push(testCoalesce_Property);


// Assignment to a super property.
function testAnd_SuperProperty(init, value) {
  let proto = {p: init};
  let obj = {__proto__: proto, m() { return (super.p &&= value); }};
  return [obj.m(), obj.p];
}
testCasesAnd.push(testAnd_SuperProperty);

function testOr_SuperProperty(init, value) {
  let proto = {p: init};
  let obj = {__proto__: proto, m() { return (super.p ||= value); }};
  return [obj.m(), obj.p];
}
testCasesOr.push(testOr_SuperProperty);

function testCoalesce_SuperProperty(init, value) {
  let proto = {p: init};
  let obj = {__proto__: proto, m() { return (super.p ??= value); }};
  return [obj.m(), obj.p];
}
testCasesCoalesce.push(testCoalesce_SuperProperty);


// Assignment to an element.
function testAnd_Element(init, value) {
  let p = 123;
  let obj = {[p]: init};
  return [(obj[p] &&= value), obj[p]];
}
testCasesAnd.push(testAnd_Element);

function testOr_Element(init, value) {
  let p = 123;
  let obj = {[p]: init};
  return [(obj[p] ||= value), obj[p]];
}
testCasesOr.push(testOr_Element);

function testCoalesce_Element(init, value) {
  let p = 123;
  let obj = {[p]: init};
  return [(obj[p] ??= value), obj[p]];
}
testCasesCoalesce.push(testCoalesce_Element);


// Assignment to a super element.
function testAnd_SuperElement(init, value) {
  let p = 123;
  let proto = {[p]: init};
  let obj = {__proto__: proto, m() { return (super[p] &&= value); }};
  return [obj.m(), obj[p]];
}
testCasesAnd.push(testAnd_SuperElement);

function testOr_SuperElement(init, value) {
  let p = 123;
  let proto = {[p]: init};
  let obj = {__proto__: proto, m() { return (super[p] ||= value); }};
  return [obj.m(), obj[p]];
}
testCasesOr.push(testOr_SuperElement);

function testCoalesce_SuperElement(init, value) {
  let p = 123;
  let proto = {[p]: init};
  let obj = {__proto__: proto, m() { return (super[p] ??= value); }};
  return [obj.m(), obj[p]];
}
testCasesCoalesce.push(testCoalesce_SuperElement);


// Run the actual tests.

function runTest(testCases, init, value, expected) {
  for (let f of testCases) {
    let [result, newValue] = f(init, value);

    assert.sameValue(result, expected);
    assert.sameValue(newValue, expected);
  }
}

function testAnd(init, value) {
  const expected = init ? value : init;
  runTest(testCasesAnd, init, value, expected);
}

function testOr(init, value) {
  const expected = !init ? value : init;
  runTest(testCasesOr, init, value, expected);
}

function testCoalesce(init, value) {
  const expected = init === undefined || init === null ? value : init;
  runTest(testCasesCoalesce, init, value, expected);
}


// Repeat a number of times to ensure JITs can kick in, too.
for (let i = 0; i < 50; ++i) {
  for (let thruthy of [true, 123, 123n, "hi", [], Symbol()]) {
    testAnd(thruthy, "pass");
    testOr(thruthy, "fail");
    testCoalesce(thruthy, "fail");
  }

  for (let falsy of [false, 0, NaN, 0n, ""]) {
    testAnd(falsy, "fail");
    testOr(falsy, "pass");
    testCoalesce(falsy, "fail");
  }

  for (let nullOrUndefined of [null, undefined]) {
    testAnd(nullOrUndefined, "fail");
    testOr(nullOrUndefined, "pass");
    testCoalesce(nullOrUndefined, "pass");
  }
}


