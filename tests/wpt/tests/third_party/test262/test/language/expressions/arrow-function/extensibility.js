// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arrow-function-definitions-runtime-semantics-evaluation
description: The [[Extensible]] slot of arrow functions
info: |
  15.3.4 Runtime Semantics: InstantiateArrowFunctionExpression
    [...]
    4. Let closure be OrdinaryFunctionCreate(%Function.prototype%, sourceText,
       ArrowParameters, ConciseBody, lexical-this, scope).

  10.2.3 OrdinaryFunctionCreate ( functionPrototype, sourceText, ParameterList, Body, thisMode, Scope )
    [...]
    3. Let F be ! OrdinaryObjectCreate(functionPrototype, internalSlotsList).

  10.1.12 OrdinaryObjectCreate ( proto [ , additionalInternalSlotsList ] )
    1. Let internalSlotsList be « [[Prototype]], [[Extensible]] ».
    2. If additionalInternalSlotsList is present, append each of its elements
       to internalSlotsList.
    3. Let O be ! MakeBasicObject(internalSlotsList).

  7.3.1 MakeBasicObject ( internalSlotsList )
    [...]
    6. If internalSlotsList contains [[Extensible]], set obj.[[Extensible]] to
       true.
---*/

assert(Object.isExtensible(() => {}));
