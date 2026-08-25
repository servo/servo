// Copyright (C) 2018 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-function.prototype.tostring
description: >
  toString of built-in Function object
info: |
  ...
  If func is a built-in Function object, then return an implementation-dependent String source code representation of func.
  The representation must have the syntax of a NativeFunction.
  ...

  NativeFunction:
    function NativeFunctionAccessor_opt IdentifierName_opt ( FormalParameters ) { [ native code ] }
  NativeFunctionAccessor :
    get
    set

includes: [nativeFunctionMatcher.js, wellKnownIntrinsicObjects.js]
features: [arrow-function, Reflect, Array.prototype.includes]
---*/

const visited = [];
function visit(ns, path) {
  if (visited.includes(ns)) {
    return;
  }
  visited.push(ns);

  if (typeof ns === 'function') {
    assertNativeFunction(ns, path);
  }
  if (typeof ns !== 'function' && (typeof ns !== 'object' || ns === null)) {
    return;
  }

  const descriptors = Object.getOwnPropertyDescriptors(ns);
  Reflect.ownKeys(descriptors)
    .forEach((name) => {
      const desc = descriptors[name];
      const p = typeof name === 'symbol'
        ? `${path}[Symbol(${name.description})]`
        : `${path}.${name}`;
      if ('value' in desc) {
        visit(desc.value, p);
      } else {
        visit(desc.get, p);
        visit(desc.set, p);
      }
    });
}

WellKnownIntrinsicObjects.forEach((intrinsic) => {
  visit(intrinsic.value, intrinsic.name);
});
assert.notSameValue(visited.length, 0);
