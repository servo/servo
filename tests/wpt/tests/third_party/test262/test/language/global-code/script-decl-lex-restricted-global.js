// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-globaldeclarationinstantiation
es6id: 15.1.8
description: >
  Let binding collision with non-configurable global property (not defined
  through a declaration)
info: |
  [...]
  5. For each name in lexNames, do
     a. If envRec.HasVarDeclaration(name) is true, throw a SyntaxError
        exception.
     b. If envRec.HasLexicalDeclaration(name) is true, throw a SyntaxError
        exception.
     c. Let hasRestrictedGlobal be ? envRec.HasRestrictedGlobalProperty(name).
     d. If hasRestrictedGlobal is true, throw a SyntaxError exception.
---*/

Object.defineProperty(this, 'test262Configurable', { configurable: true });
Object.defineProperty(this, 'test262NonConfigurable', { configurable: false });

$262.evalScript('let test262Configurable;');

assert.throws(SyntaxError, function() {
  $262.evalScript('var x; let test262NonConfigurable;');
});

assert.throws(ReferenceError, function() {
  x;
});
