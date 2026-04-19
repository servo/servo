// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 25.2
description: >
    Generator function instances are correctly reported as instances of the
    GeneratorFunction intrinsic.
features: [generators]
---*/

var GeneratorFunction = Object.getPrototypeOf(function*() {}).constructor;

function* gDecl() {}
var gExpr = function*() {};

assert(
  gDecl instanceof GeneratorFunction,
  'Generators created via GeneratorDeclaration syntax are proper instances of GeneratorFunction'
);

assert(
  gExpr instanceof GeneratorFunction,
  'Generators created via GeneratorExpression syntax are proper instances of GeneratorFunction'
);

assert(
  new GeneratorFunction() instanceof GeneratorFunction,
  'Generators created via constructor invocation of GeneratorFunction are proper instances of GeneratorFunction'
);

assert(
  GeneratorFunction() instanceof GeneratorFunction,
  'Generators created via function invocation of GeneratorFunction are proper instances of GeneratorFunction'
);
