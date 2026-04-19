// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.seal
description: >
    Object.seal Set
info: |
  If Type(O) is not Object, return O.
  Let status be ? SetIntegrityLevel(O, sealed).

  SetIntegrityLevel ( O, level )

  Assert: Type(O) is Object.
  Assert: level is either sealed or frozen.
  Let status be ? O.[[PreventExtensions]]().
  If status is false, return false.
  Let keys be ? O.[[OwnPropertyKeys]]().
  If level is sealed, then
    For each element k of keys, do
      Perform ? DefinePropertyOrThrow(O, k, PropertyDescriptor { [[Configurable]]: false }).
  Else,
    Assert: level is frozen.
    For each element k of keys, do
      Let currentDesc be ? O.[[GetOwnProperty]](k).
      If currentDesc is not undefined, then
        If IsAccessorDescriptor(currentDesc) is true, then
          Let desc be the PropertyDescriptor { [[Configurable]]: false }.
        Else,
          Let desc be the PropertyDescriptor { [[Configurable]]: false, [[Writable]]: false }.
        Perform ? DefinePropertyOrThrow(O, k, desc).
  Return true.

---*/

Object.seal(new Set());
