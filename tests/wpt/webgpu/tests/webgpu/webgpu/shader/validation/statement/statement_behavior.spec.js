/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Test statement behavior analysis.

Functions must have a behavior of {Return}, {Next}, or {Return, Next}.
Functions with a return type must have a behavior of {Return}.

Each statement in the function must be valid according to the table.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

const kInvalidStatements = {
  break: `break`,
  break_if: `break if true`,
  continue: `continue`,
  loop1: `loop { }`,
  loop2: `loop { continuing { } }`,
  loop3: `loop { continue; continuing { } }`,
  loop4: `loop { continuing { break; } }`,
  loop5: `loop { continuing { continue; } }`,
  loop6: `loop { continuing { return; } }`,
  loop7: `loop { continue; break; }`,
  loop8: `loop { continuing { break if true; return; } }`,
  for1: `for (;;) { }`,
  for2: `for (var i = 0; ; i++) { }`,
  for3: `for (;; break) { }`,
  for4: `for (;; continue ) { }`,
  for5: `for (;; return ) { }`,
  for6: `for (;;) { continue; break; }`,
  // while loops always have break in their behaviors.
  switch1: `switch (1) { case 1 { } }`,
  sequence1: `return; loop { }`,
  compound1: `{ loop { } }`
};

g.test('invalid_statements').
desc('Test statements with invalid behaviors').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#behaviors-rules').
params((u) => u.combine('body', keysOf(kInvalidStatements))).
fn((t) => {
  const body = kInvalidStatements[t.params.body];
  const code = `fn foo() {
      ${body};
    }`;
  t.expectCompileResult(false, code);
});

const kValidStatements = {
  empty: ``,
  const_assert: `const_assert true`,
  let: `let x = 1`,
  var1: `var x = 1`,
  var2: `var x : i32`,
  assign: `v = 1`,
  phony_assign: `_ = 1`,
  compound_assign: `v += 1`,
  return: `return`,
  discard: `discard`,
  function_call1: `bar()`,
  function_call2: `workgroupBarrier()`,

  if1: `if true { } else { }`,
  if2: `if true { }`,

  break1: `loop { break; }`,
  break2: `loop { if false { break; } }`,
  break_if: `loop { continuing { break if false; } }`,

  continue1: `loop { continue; continuing { break if true; } }`,

  loop1: `loop { break; }`,
  loop2: `loop { break; continuing { } }`,
  loop3: `loop { continue; continuing { break if true; } }`,
  loop4: `loop { break; continue; }`,

  for1: `for (; true; ) { }`,
  for2: `for (;;) { break; }`,
  for3: `for (;true;) { continue; }`,

  while1: `while true { }`,
  while2: `while true { continue; }`,
  while3: `while true { continue; break; }`,

  switch1: `switch 1 { default { } }`,
  switch2: `switch 1 { case 1 { } default { } }`,
  switch3: `switch 1 { default { break; } }`,
  switch4: `switch 1 { default { } case 1 { break; } }`,

  sequence1: `return; let x = 1`,
  sequence2: `if true { } let x = 1`,
  sequence3: `switch 1 { default { break; return; } }`,

  compound1: `{ }`,
  compound2: `{ loop { break; } if true { return; } }`
};

g.test('valid_statements').
desc('Test statements with valid behaviors').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#behaviors-rules').
params((u) => u.combine('body', keysOf(kValidStatements))).
fn((t) => {
  const body = kValidStatements[t.params.body];
  const code = `
    var<private> v : i32;
    fn bar() { }
    fn foo() {
      ${body};
    }`;
  t.expectCompileResult(true, code);
});

const kInvalidFunctions = {
  next_for_type: `fn foo() -> bool { }`,
  next_return_for_type: `fn foo() -> bool { if true { return true; } }`
};

g.test('invalid_functions').
desc('Test functions with invalid behaviors').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#behaviors-rules').
params((u) => u.combine('function', keysOf(kInvalidFunctions))).
fn((t) => {
  const func = kInvalidFunctions[t.params.function];
  t.expectCompileResult(false, func);
});

const kValidFunctions = {
  empty: `fn foo() { }`,
  next_return: `fn foo() { if true { return; } }`,
  unreachable_code_after_return_with_value: `fn foo() -> bool { return false; _ = 0; }`,
  no_final_return: `fn foo() -> bool { if true { return true; } else { return false; } }`,
  no_final_return_unreachable_code: `fn foo() -> bool { if true { return true; } else { return false; } _ = 0; _ = 1; }`
};

g.test('valid_functions').
desc('Test functions with valid behaviors').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#behaviors-rules').
params((u) => u.combine('function', keysOf(kValidFunctions))).
fn((t) => {
  const func = kValidFunctions[t.params.function];
  t.expectCompileResult(true, func);
});