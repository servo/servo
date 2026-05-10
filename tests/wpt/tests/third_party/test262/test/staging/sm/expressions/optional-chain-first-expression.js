// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// Verify bytecode emitter accepts all valid optional chain first expressions.

const expressions = [
  // https://tc39.es/ecma262/#sec-primary-expression
  "this",
  "ident",
  "null",
  "true",
  "false",
  "123",
  "123n",
  "'str'",
  "[]",
  "{}",
  "function(){}",
  "class{}",
  "function*(){}",
  "async function(){}",
  "async function*(){}",
  "/a/",
  "`str`",
  "(a + b)",

  // https://tc39.es/ecma262/#sec-left-hand-side-expressions
  "a[b]",
  "a.b",
  "a``",
  "super[a]",
  "super.a",
  "new.target",
  "import.meta",
  "new C()",
  "new C",
  "f()",
  "super()",
  "a?.b",
  "a?.[b]",
  "a?.()",
  "a?.``",
];

function tryRun(s) {
  try { Function(s)(); } catch {}
}

for (let expr of expressions) {
  // Evaluate in an expression context.
  tryRun(`void (${expr}?.());`);
  tryRun(`void (${expr}?.p());`);

  // Also try parenthesized.
  tryRun(`void ((${expr})?.());`);
  tryRun(`void ((${expr})?.p());`);
}

function inClassConstructor(s) {
  return `class C { constructor() { ${s} } }`;
}

for (let expr of ["super[a]", "super.a", "super()"]) {
  // Evaluate in an expression context.
  tryRun(inClassConstructor(`void (${expr}?.());`));
  tryRun(inClassConstructor(`void (${expr}?.p());`));

  // Also try parenthesized.
  tryRun(inClassConstructor(`void ((${expr})?.());`));
  tryRun(inClassConstructor(`void ((${expr})?.p());`));
}
