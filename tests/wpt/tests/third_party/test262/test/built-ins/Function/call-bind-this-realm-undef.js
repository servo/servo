// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-ecmascript-function-objects-call-thisargument-argumentslist
description: The "this" value is set to the global This value
info: |
  [...]
  6. Perform OrdinaryCallBindThis(F, calleeContext, thisArgument).
  [...]

  9.2.1.2OrdinaryCallBindThis ( F, calleeContext, thisArgument )#

  [...]
  5. If thisMode is strict, let thisValue be thisArgument.
  6. Else,
     a. If thisArgument is null or undefined, then
        i. Let globalEnv be calleeRealm.[[GlobalEnv]].
        ii. Let globalEnvRec be globalEnv's EnvironmentRecord.
        iii. Let thisValue be globalEnvRec.[[GlobalThisValue]].
  [...]
features: [cross-realm]
---*/

var other = $262.createRealm().global;
var func = new other.Function('return this;');
var subject;

assert.sameValue(func(), other, 'implicit undefined');
assert.sameValue(func.call(undefined), other, 'explicit undefined');
assert.sameValue(func.call(null), other, 'null');
