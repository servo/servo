/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Validation tests for @must_use`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

const kMustUseDeclarations = {
  var: {
    code: `@must_use @group(0) @binding(0)
    var<storage> x : array<u32>;`,
    valid: false
  },
  function_no_return: {
    code: `@must_use fn foo() { }`,
    valid: false
  },
  function_scalar_return: {
    code: `@must_use fn foo() -> u32 { return 0; }`,
    valid: true
  },
  function_struct_return: {
    code: `struct S { x : u32 }
    @must_use fn foo() -> S { return S(); }`,
    valid: true
  },
  function_var: {
    code: `fn foo() { @must_use var x = 0; }`,
    valid: false
  },
  function_call: {
    code: `fn bar() -> u32 { return 0; }
    fn foo() { @must_use bar(); }`,
    valid: false
  },
  function_parameter: {
    code: `fn foo(@must_use param : u32) -> u32 { return param; }`,
    valid: false
  },
  empty_parameter: {
    code: `@must_use() fn foo() -> u32 { return 0; }`,
    valid: false
  },
  parameter: {
    code: `@must_use(0) fn foo() -> u32 { return 0; }`,
    valid: false
  },
  duplicate: {
    code: `@must_use @must_use fn foo() -> u32 { return 0; }`,
    valid: false
  }
};

g.test('declaration').
desc(`Validate attribute can only be applied to a function declaration with a return type`).
params((u) => u.combine('test', keysOf(kMustUseDeclarations))).
fn((t) => {
  const test = kMustUseDeclarations[t.params.test];
  t.expectCompileResult(test.valid, test.code);
});

const kMustUseCalls = {
  no_call: ``, // Never calling a @must_use function should pass
  phony: `_ = bar();`,
  let: `let tmp = bar();`,
  local_var: `var tmp = bar();`,
  private_var: `private_var = bar();`,
  storage_var: `storage_var = bar();`,
  pointer: `
    var a : f32;
    let p = &a;
    (*p) = bar();`,
  vector_elem: `
    var a : vec3<f32>;
    a.x = bar();`,
  matrix_elem: `
    var a : mat3x2<f32>;
    a[0][0] = bar();`,
  condition: `if bar() == 0 { }`,
  param: `baz(bar());`,
  return: `return bar();`,
  statement: `bar();` // should fail if bar is @must_use
};

g.test('call').
desc(`Validate that a call to must_use function cannot be the whole function call statement`).
params((u) =>
u //
.combine('use', ['@must_use', '']).
combine('call', keysOf(kMustUseCalls))
).
fn((t) => {
  const test = kMustUseCalls[t.params.call];
  const code = `
    @group(0) @binding(0) var<storage, read_write> storage_var : f32;
    var<private> private_var : f32;

    fn baz(param : f32) { }

    ${t.params.use} fn bar() -> f32 { return 0; }

    fn foo() ${t.params.call === 'return' ? '-> f32' : ''} {
      ${test}
    }`;

  const should_pass = t.params.call !== 'statement' || t.params.use === '';
  t.expectCompileResult(should_pass, code);
});

g.test('ignore_result_of_non_must_use_that_returns_call_of_must_use').
desc(
  `Test that ignoring the result of a non-@must_use function that returns the result of a @must_use function succeeds`
).
fn((t) => {
  const wgsl = `
    @must_use
    fn f() -> f32 {
      return 0;
    }

    fn g() -> f32 {
      return f();
    }

    fn main() {
      g(); // Ignore result
    }
    `;

  t.expectCompileResult(true, wgsl);
});