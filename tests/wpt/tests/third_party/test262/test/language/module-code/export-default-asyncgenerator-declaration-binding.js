// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
  ExportDeclaration : HoistableDeclaration : AsyncGeneratorDeclaration
  esid: prod-HoistableDeclaration
info: |
  ExportDeclaration :
    HoistableDeclaration[Yield, Await, Default]:

  HoistableDeclaration[Yield, Await, Default]:
    AsyncGeneratorDeclaration[?Yield, ?Await, ?Default]

flags: [module]
---*/

export default async function * AG() {}
AG.foo = '';
