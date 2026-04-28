// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-runtime-semantics-propertydestructuringassignmentevaluation
description: >
  Ensure correct evaluation order for binding lookups when destructuring target is var-binding.
info: |
  13.15.5.3 Runtime Semantics: PropertyDestructuringAssignmentEvaluation

    AssignmentProperty : PropertyName : AssignmentElement

    1. Let name be ? Evaluation of PropertyName.
    2. Perform ? KeyedDestructuringAssignmentEvaluation of AssignmentElement with arguments value and name.
    ...

  13.15.5.6 Runtime Semantics: KeyedDestructuringAssignmentEvaluation

    AssignmentElement : DestructuringAssignmentTarget Initializer_opt

    1. If DestructuringAssignmentTarget is neither an ObjectLiteral nor an ArrayLiteral, then
      a. Let lRef be ? Evaluation of DestructuringAssignmentTarget.
    2. Let v be ? GetV(value, propertyName).
    3. If Initializer is present and v is undefined, then
      ...
      b. Else,
        i. Let defaultValue be ? Evaluation of Initializer.
        ii. Let rhsValue be ? GetValue(defaultValue).
    ...
    6. Return ? PutValue(lRef, rhsValue).

includes: [compareArray.js]
features: [Proxy]
flags: [noStrict]
---*/

var log = [];

var targetKey = {
  toString: () => {
    log.push("targetKey");
    return "q";
  }
};

var sourceKey = {
  toString: () => {
    log.push("sourceKey");
    return "p";
  }
};

var source = {
  get p() {
    log.push("get source");
    return undefined;
  }
};

var target = {
  set q(v) {
    log.push("set target");
  },
};

var env = new Proxy({}, {
  has(t, pk) {
    log.push("binding::" + pk);
  }
});

var defaultValue = 0;

with (env) {
  ({
    [sourceKey]: target[targetKey] = defaultValue
  } = source);
}

assert.compareArray(log, [
  "binding::source",
  "binding::sourceKey",
  "sourceKey",
  "binding::target",
  "binding::targetKey",
  "get source",
  "binding::defaultValue",
  "targetKey",
  "set target",
]);
