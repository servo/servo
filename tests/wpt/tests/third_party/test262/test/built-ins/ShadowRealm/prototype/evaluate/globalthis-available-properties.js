// Copyright (C) 2021 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-shadowrealm.prototype.evaluate
description: >
  The ShadowRealm global must include ECMAScript global properties
info: |
  ShadowRealm ( )

  ...
  3. Let realmRec be CreateRealm().
  4. Set O.[[ShadowRealm]] to realmRec.
  ...
  10. Perform ? SetRealmGlobalObject(realmRec, undefined, undefined).
  11. Perform ? SetDefaultGlobalBindings(O.[[ShadowRealm]]).
  12. Perform ? HostInitializeShadowRealm(O.[[ShadowRealm]]).

  SetDefaultGlobalBindings ( realmRec )

  1. Let global be realmRec.[[GlobalObject]].
  2. For each property of the Global Object specified in clause 19, do
    a. Let name be the String value of the property name.
    b. Let desc be the fully populated data Property Descriptor for the property, containing the specified attributes for the property. For properties listed in 19.2, 19.3, or 19.4 the value of the [[Value]] attribute is the corresponding intrinsic object from realmRec.
    c. Perform ? DefinePropertyOrThrow(global, name, desc).
  3. Return global.
features: [ShadowRealm]
includes: [compareArray.js]
---*/

assert.sameValue(
  typeof ShadowRealm.prototype.evaluate,
  'function',
  'This test must fail if ShadowRealm.prototype.evaluate is not a function'
);

const r = new ShadowRealm();

let properties = [
  'globalThis',
  'Infinity',
  'NaN',
  'undefined',
  'eval',
  'isFinite',
  'isNaN',
  'parseFloat',
  'parseInt',
  'decodeURI',
  'decodeURIComponent',
  'encodeURI',
  'encodeURIComponent',
  'AggregateError',
  'Array',
  'ArrayBuffer',
  'BigInt',
  'BigInt64Array',
  'BigUint64Array',
  'Boolean',
  'DataView',
  'Date',
  'Error',
  'EvalError',
  'FinalizationRegistry',
  'Float16Array',
  'Float32Array',
  'Float64Array',
  'Function',
  'Int8Array',
  'Int16Array',
  'Int32Array',
  'Map',
  'Number',
  'Object',
  'Promise',
  'Proxy',
  'RangeError',
  'ReferenceError',
  'RegExp',
  'Set',
  'SharedArrayBuffer',
  'String',
  'Symbol',
  'SyntaxError',
  'TypeError',
  'Uint8Array',
  'Uint8ClampedArray',
  'Uint16Array',
  'Uint32Array',
  'URIError',
  'WeakMap',
  'WeakRef',
  'WeakSet',
  'Atomics',
  'JSON',
  'Math',
  'Reflect',
];

// The intention of this test is to ensure that all built-in properties of the
// global object are also exposed on the ShadowRealm's global object, without
// penalizing implementations that don't have all of them implemented. Notably,
// SharedArrayBuffer may still not be (re-)enabled in all circumstances.
properties = properties.filter(name => {
    return name in globalThis;
});

const available = properties.filter(name => {
  // This test is intentionally not using wrapped functions.
  // This test should not depend on wrapped functions.
  return r.evaluate(`Object.prototype.hasOwnProperty.call(globalThis, '${name}')`);
});

// This comparison is intentional to list difference in names if the the assertion fails
assert.compareArray(properties, available);
