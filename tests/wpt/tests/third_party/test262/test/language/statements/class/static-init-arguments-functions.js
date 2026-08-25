// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-class-definitions-static-semantics-early-errors
description: The identifier `arguments` is not restricted within function forms
info: |
  ClassStaticBlockBody : ClassStaticBlockStatementList

  - It is a Syntax Error if ContainsArguments of ClassStaticBlockStatementList
    is true.
includes: [compareArray.js]
features: [class-static-block]
---*/

var fn, fnParam;
var gen, genParam;
var asyncFn, asyncFnParam;

class C {
  static {
    (function({test262 = fnParam = arguments}) {
      fn = arguments;
    })('function');

    (function * ({test262 = genParam = arguments}) {
      gen = arguments;
    })('generator function').next();

    (async function ({test262 = asyncFnParam = arguments}) {
      asyncFn = arguments;
    })('async function');
  }
}

assert.compareArray(['function'], fn, 'body');
assert.compareArray(['function'], fnParam, 'parameter');
assert.compareArray(['generator function'], gen, 'body');
assert.compareArray(['generator function'], genParam, 'parameter');
assert.compareArray(['async function'], asyncFn, 'body');
assert.compareArray(['async function'], asyncFnParam, 'parameter');
