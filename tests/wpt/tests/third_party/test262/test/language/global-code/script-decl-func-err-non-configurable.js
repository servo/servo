// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-globaldeclarationinstantiation
es6id: 15.1.8
description: >
  Declaration of function when there is a corresponding global property that is
  non-configurable but *not* a writable and configurable data property.
info: |
  [...]
  9. Let declaredFunctionNames be a new empty List.
  10. For each d in varDeclarations, in reverse list order do
      a. If d is neither a VariableDeclaration or a ForBinding, then
         i. Assert: d is either a FunctionDeclaration or a
            GeneratorDeclaration.
         ii. NOTE If there are multiple FunctionDeclarations for the same name,
             the last declaration is used.
         iii. Let fn be the sole element of the BoundNames of d.
         iv. If fn is not an element of declaredFunctionNames, then
             1. Let fnDefinable be ? envRec.CanDeclareGlobalFunction(fn).
             2. If fnDefinable is false, throw a TypeError exception.

  8.1.1.4.16 CanDeclareGlobalFunction

  [...]
  6. If existingProp.[[Configurable]] is true, return true.
  7. If IsDataDescriptor(existingProp) is true and existingProp has attribute
     values {[[Writable]]: true, [[Enumerable]]: true}, return true.
  8. Return false. 
---*/

Object.defineProperty(
  this,
  'data1',
  { configurable: false, value: 0, writable: true, enumerable: false }
);

Object.defineProperty(
  this,
  'data2',
  { configurable: false, value: 0, writable: false, enumerable: true }
);

Object.defineProperty(
  this,
  'data3',
  { configurable: false, value: 0, writable: false, enumerable: false }
);

Object.defineProperty(
  this,
  'accessor1',
  { 
    configurable: false,
    get: function() {},
    set: function() {},
    enumerable: true
  }
);

Object.defineProperty(
  this,
  'accessor2',
  { 
    configurable: false,
    get: function() {},
    set: function() {},
    enumerable: true
  }
);

assert.throws(TypeError, function() {
  $262.evalScript('var x; function data1() {}');
}, 'writable, non-enumerable data property');
assert.throws(ReferenceError, function() {
  x;
}, 'bindings not created for writable, non-enumerable data property');

assert.throws(TypeError, function() {
  $262.evalScript('var x; function data2() {}');
}, 'non-writable, enumerable data property');
assert.throws(ReferenceError, function() {
  x;
}, 'bindings not created for non-writable, enumerable data property');

assert.throws(TypeError, function() {
  $262.evalScript('var x; function data3() {}');
}, 'non-writable, non-enumerable data property');
assert.throws(ReferenceError, function() {
  x;
}, 'bindings not created for non-writable, non-enumerable data property');

assert.throws(TypeError, function() {
  $262.evalScript('var x; function accessor1() {}');
}, 'enumerable accessor property');
assert.throws(ReferenceError, function() {
  x;
}, 'bindings not created for enumerableaccessor property');

assert.throws(TypeError, function() {
  $262.evalScript('var x; function accessor2() {}');
}, 'non-enumerable accessor property');
assert.throws(ReferenceError, function() {
  x;
}, 'bindings not created for non-enumerableaccessor property');
