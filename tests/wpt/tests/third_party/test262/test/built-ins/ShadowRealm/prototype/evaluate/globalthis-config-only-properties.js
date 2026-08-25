// Copyright (C) 2021 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-shadowrealm.prototype.evaluate
description: >
  Properties of globalThis must be configurable
info: |
  ShadowRealm ( )

  ...
  3. Let realmRec be CreateRealm().
  4. Set O.[[ShadowRealm]] to realmRec.
  ...
  10. Perform ? SetRealmGlobalObject(realmRec, undefined, undefined).
  11. Perform ? SetDefaultGlobalBindings(O.[[ShadowRealm]]).
  12. Perform ? HostInitializeShadowRealm(O.[[ShadowRealm]]).

  Runtime Semantics: HostInitializeShadowRealm ( realm )

  HostInitializeShadowRealm is an implementation-defined abstract operation
  used to inform the host of any newly created realms from the ShadowRealm
  constructor. Its return value is not used, though it may throw an exception.
  The idea of this hook is to initialize host data structures related to the
  ShadowRealm, e.g., for module loading.

  The host may use this hook to add properties to the ShadowRealm's global
  object. Those properties must be configurable.
features: [ShadowRealm, Array.prototype.includes]
---*/

assert.sameValue(
  typeof ShadowRealm.prototype.evaluate,
  'function',
  'This test must fail if ShadowRealm.prototype.evaluate is not a function'
);

const r = new ShadowRealm();

const anyMissed = r.evaluate(`
  // These names are the only exception as non configurable values.
  // Yet, they don't represent any object value.
  const esNonConfigValues = [
    'undefined',
    'Infinity',
    'NaN'
  ];

  const entries = Object.entries(Object.getOwnPropertyDescriptors(globalThis));

  const missed = entries
    .filter(entry => entry[1].configurable === false)
    .map(([name]) => name)
    .filter(name => !esNonConfigValues.includes(name))
    .join(', ');

  missed;
`);

assert.sameValue(anyMissed, '', 'All globalThis properties must be configurable');

const result = r.evaluate(`
  const ObjectKeys = Object.keys;
  const hasOwn = Object.prototype.hasOwnProperty;
  const savedGlobal = globalThis;
  const names = Object.keys(Object.getOwnPropertyDescriptors(globalThis));

  // These names are the only exception as non configurable values.
  // Yet, they don't represent any object value.
  const esNonConfigValues = [
    'undefined',
    'Infinity',
    'NaN'
  ];

  // Delete every name except globalThis, for now
  const remainingNames = names.filter(name => {
    if (esNonConfigValues.includes(name)) {
      return false;
    }

    if (name !== 'globalThis') {
      delete globalThis[name];
      return hasOwn.call(globalThis, name);
    }
  });

  delete globalThis['globalThis'];

  if (hasOwn.call(savedGlobal, 'globalThis')) {
    remainingNames.push('globalThis');
  }

  const failedDelete = remainingNames.join(', ');

  failedDelete;
`);

assert.sameValue(result, '', 'deleting any globalThis property must be effective');
