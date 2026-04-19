// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-class-definitions-static-semantics-early-errors
description: The identifier `arguments` is not restricted within method forms
info: |
  ClassStaticBlockBody : ClassStaticBlockStatementList

  - It is a Syntax Error if ContainsArguments of ClassStaticBlockStatementList
    is true.
includes: [compareArray.js]
features: [class-static-block]
---*/

var instance;
var method, methodParam;
var getter;
var setter, setterParam;
var genMethod, genMethodParam;
var asyncMethod, asyncMethodParam;

class C {
  static {
    instance = new class {
      method({test262 = methodParam = arguments}) {
        method = arguments;
      }
      get accessor() {
        getter = arguments;
      }
      set accessor({test262 = setterParam = arguments}) {
        setter = arguments;
      }
      *gen({test262 = genMethodParam = arguments}) {
        genMethod = arguments;
      }
      async async({test262 = asyncMethodParam = arguments}) {
        asyncMethod = arguments;
      }
    }();
  }
}

instance.method('method');
instance.accessor;
instance.accessor = 'setter';
instance.gen('generator method').next();
instance.async('async method');

assert.compareArray(['method'], method, 'body');
assert.compareArray(['method'], methodParam, 'parameter');
assert.compareArray([], getter, 'body');
assert.compareArray(['setter'], setter, 'body');
assert.compareArray(['setter'], setterParam, 'parameter');
assert.compareArray(['generator method'], genMethod, 'body');
assert.compareArray(['generator method'], genMethodParam, 'parameter');
assert.compareArray(['async method'], asyncMethod, 'body');
assert.compareArray(['async method'], asyncMethodParam, 'parameter');
