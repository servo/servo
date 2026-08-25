// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 12.2.8.5
description: Evaluation of NoSubstitutionTemplate
info: |
    12.2.8.5 Runtime Semantics: Evaluation
    TemplateLiteral : NoSubstitutionTemplate

    1. Return the string value whose code units are the elements of the TV of
       NoSubstitutionTemplate as defined in 11.8.6.
---*/

assert.sameValue(`NoSubstitutionTemplate`, 'NoSubstitutionTemplate');
