// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-globaldeclarationinstantiation
es6id: 15.1.8
description: >
  Declaration of function when there is no corresponding global property and
  the global object is non-extensible
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

  1. Let envRec be the global Environment Record for which the method was
     invoked.
  2. Let ObjRec be envRec.[[ObjectRecord]].
  3. Let globalObject be the binding object for ObjRec.
  4. Let existingProp be ? globalObject.[[GetOwnProperty]](N).
  5. If existingProp is undefined, return ? IsExtensible(globalObject).
---*/

var executed = false;

Object.preventExtensions(this);

assert.throws(TypeError, function() {
  $262.evalScript('executed = true; function test262() {}');
});

assert.sameValue(executed, false);
