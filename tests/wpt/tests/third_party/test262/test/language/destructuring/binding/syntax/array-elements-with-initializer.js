// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 13.3.3
description: >
  The ArrayBindingPattern with an element list with initializers
info: |
  Destructuring Binding Patterns - Syntax

  ArrayBindingPattern[Yield] :
    [ Elisionopt BindingRestElement[?Yield]opt ]
    [ BindingElementList[?Yield] ]
    [ BindingElementList[?Yield] , Elisionopt BindingRestElement[?Yield]opt ]

  BindingElementList[Yield] :
    BindingElisionElement[?Yield]
    BindingElementList[?Yield] , BindingElisionElement[?Yield]

  BindingElisionElement[Yield] :
    Elisionopt BindingElement[?Yield]

  BindingElement[Yield ] :
    SingleNameBinding[?Yield]
    BindingPattern[?Yield] Initializer[In, ?Yield]opt
features: [destructuring-binding]
---*/

function fn1([a, b = 42]) {}

function fn2([a = 42, b,]) {}

function fn3([a,, b = a, c = 42]) {}
