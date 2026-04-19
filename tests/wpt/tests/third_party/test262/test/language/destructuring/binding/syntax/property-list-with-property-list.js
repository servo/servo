// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 13.3.3
description: >
  The ObjectBindingPattern with deep binding property lists
info: |
  Destructuring Binding Patterns - Syntax

  ObjectBindingPattern[Yield] :
    { }
    { BindingPropertyList[?Yield] }
    { BindingPropertyList[?Yield] , }

  BindingPropertyList[Yield] :
    BindingProperty[?Yield]
    BindingPropertyList[?Yield] , BindingProperty[?Yield]

features: [destructuring-binding]
---*/

function fn1({a: {p: q}, b: {r}, c: {s = 0}, d: {}}) {}

function fn2(x, {a: r, b: s, c: t}, y) {}

function fn3({x: {y: {z: {} = 42}}}) {}
