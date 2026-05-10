// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-properties-of-asyncgeneratorfunction
description: >
    AsyncGenerator function instances are correctly reported as instances of the
    AsyncGeneratorFunction intrinsic.
features: [async-iteration]
---*/

var AsyncGeneratorFunction = Object.getPrototypeOf(async function* () {}).constructor;

async function* agDecl() {}
var agExpr = async function* () {};

assert(
  agDecl instanceof AsyncGeneratorFunction,
  'AsyncGenerators created via AsyncGeneratorDeclaration syntax are proper instances of AsyncGeneratorFunction'
);

assert(
  agExpr instanceof AsyncGeneratorFunction,
  'AsyncGenerators created via AsyncGeneratorExpression syntax are proper instances of AsyncGeneratorFunction'
);

assert(
  new AsyncGeneratorFunction() instanceof AsyncGeneratorFunction,
  'AsyncGenerators created via constructor invocation of AsyncGeneratorFunction are proper instances of AsyncGeneratorFunction'
);

assert(
  AsyncGeneratorFunction() instanceof AsyncGeneratorFunction,
  'AsyncGenerators created via function invocation of AsyncGeneratorFunction are proper instances of AsyncGeneratorFunction'
);
