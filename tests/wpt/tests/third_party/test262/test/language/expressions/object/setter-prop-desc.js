// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-object-initializer-runtime-semantics-evaluation
es6id: 12.2.6.8
description: Property descriptor of "set" accessor methods
info: |
  ObjectLiteral:
    { PropertyDefinitionList }
    { PropertyDefinitionList , }

  1. Let obj be ObjectCreate(%ObjectPrototype%).
  2. Let status be the result of performing PropertyDefinitionEvaluation of
     PropertyDefinitionList with arguments obj and true.
  3. ReturnIfAbrupt(status).
  4. Return obj. 

  14.3.8 Runtime Semantics: PropertyDefinitionEvaluation

  MethodDefinition : set PropertyName ( PropertySetParameterList ) { FunctionBody }

  [...]
  8. Let desc be the PropertyDescriptor{[[Set]]: closure, [[Enumerable]]:
     enumerable, [[Configurable]]: true}.
  [...]
includes: [propertyHelper.js]
---*/

var obj = { set m(x) { return x; } };
var desc = Object.getOwnPropertyDescriptor(obj, 'm');

verifyProperty(obj, 'm', {
  enumerable: true,
  configurable: true,
});

assert.sameValue(desc.value, undefined, '`value` field');
assert.sameValue(desc.get, undefined, '`get` field');
assert.sameValue(typeof desc.set, 'function', 'type of `set` field');
assert.sameValue(desc.set(436), 436, '`set` function return value');
