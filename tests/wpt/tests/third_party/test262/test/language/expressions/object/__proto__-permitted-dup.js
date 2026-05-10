// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-object-initializer
description: Permitted duplicate `__proto__` property
info: |
    Annex B defines an early error for duplicate PropertyName of `__proto__`,
    but this does not apply to properties created from other productions.

    B.3.1 __proto__ Property Names in Object Initializers

    It is a Syntax Error if PropertyNameList of PropertyDefinitionList contains
    any duplicate entries for "__proto__" and at least two of those entries
    were obtained from productions of the form
    PropertyDefinition : PropertyName : AssignmentExpression .
features: [generators, async-functions, async-iteration, __proto__]
---*/

var obj = {
  __proto__: null,
  __proto_: null,
  __proto: null,
  _proto__: null,
  proto__: null,
  ['__proto__']: null,
  __proto__() {},
  * __proto__() {},
  async __proto__() {},
  async * __proto__() {},
  get __proto__() { return 33; },
  set __proto__(_) { return 44; }
};

var desc = Object.getOwnPropertyDescriptor(obj, '__proto__');

assert.sameValue(desc.get(), 33);
assert.sameValue(desc.set(), 44);
