// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 13.3.3
description: >
  The ArrayBindingPattern with elisions only
info: |
  Destructuring Binding Patterns - Syntax

  ArrayBindingPattern[Yield] :
    [ Elisionopt BindingRestElement[?Yield]opt ]
    [ BindingElementList[?Yield] ]
    [ BindingElementList[?Yield] , Elisionopt BindingRestElement[?Yield]opt ]
features: [destructuring-binding]
---*/

function fn1([,]) {}
function fn2([,,]) {}
