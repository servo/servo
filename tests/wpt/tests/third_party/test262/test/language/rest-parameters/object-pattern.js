// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-destructuring-binding-patterns
description: >
  The rest parameter can be a binding pattern.
info: |
  Destructuring Binding Patterns - Syntax

  BindingRestElement[Yield]:
    ...BindingPattern[?Yield]
---*/

function empty(...{}) {}

function emptyWithArray(...{p: []}) {}

function emptyWithObject(...{p: {}}) {}

function emptyWithLeading(x, ...{}) {}


function singleElement(...{a: b}) {}

function singleElementWithInitializer(...{a: b = 0}) {}

function singleElementWithArray(...{p: [a]}) {}

function singleElementWithObject(...{p: {a: b}}) {}

function singleElementWithLeading(x, ...{a: b}) {}


function multiElement(...{a: r, b: s, c: t}) {}

function multiElementWithInitializer(...{a: r = 0, b: s, c: t = 1}) {}

function multiElementWithArray(...{p: [a], b, q: [c]}) {}

function multiElementWithObject(...{a: {p: q}, b: {r}, c: {s = 0}}) {}

function multiElementWithLeading(x, y, ...{a: r, b: s, c: t}) {}

