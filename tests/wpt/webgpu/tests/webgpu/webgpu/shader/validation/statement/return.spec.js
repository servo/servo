/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Validation tests for 'return' statements'`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { isConvertible, scalarTypeOf, Type } from '../../../util/conversion.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

const kTestTypesNoAbstract = [
'bool',
'i32',
'u32',
'f32',
'f16',
'vec2f',
'vec3h',
'vec4u',
'vec3b',
'mat2x3f',
'mat4x2h'];


const kTestTypes = [
...kTestTypesNoAbstract,
'abstract-int',
'abstract-float',
'vec2af',
'vec3af',
'vec4af',
'vec2ai',
'vec3ai',
'vec4ai'];


g.test('return_missing_value').
desc(`Tests that a 'return' must have a value if the function has a return type`).
params((u) => u.combine('type', [...kTestTypesNoAbstract, undefined])).
beforeAllSubcases((t) => {
  if (t.params.type !== undefined && scalarTypeOf(Type[t.params.type]).kind) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const type = t.params.type ? Type[t.params.type] : undefined;
  const enable = type && scalarTypeOf(type).kind === 'f16' ? 'enable f16;' : '';
  const code = `
${enable}

fn f()${type ? `-> ${type}` : ''} {
  return;
}
`;

  const pass = type === undefined;
  t.expectCompileResult(pass, code);
});

g.test('return_unexpected_value').
desc(`Tests that a 'return' must not have a value if the function has no return type`).
params((u) => u.combine('type', [...kTestTypes, undefined])).
beforeAllSubcases((t) => {
  if (t.params.type !== undefined && scalarTypeOf(Type[t.params.type]).kind) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const type = t.params.type ? Type[t.params.type] : undefined;
  const enable = type && scalarTypeOf(type).kind === 'f16' ? 'enable f16;' : '';
  const code = `
${enable}

fn f() {
  return ${type ? `${type.create(1).wgsl()}` : ''};
}
`;

  const pass = type === undefined;
  t.expectCompileResult(pass, code);
});

g.test('return_type_match').
desc(`Tests that a 'return' value type must match the function return type`).
params((u) =>
u.combine('return_value_type', kTestTypes).combine('fn_return_type', kTestTypesNoAbstract)
).
beforeAllSubcases((t) => {
  if (
  scalarTypeOf(Type[t.params.return_value_type]).kind === 'f16' ||
  scalarTypeOf(Type[t.params.fn_return_type]).kind === 'f16')
  {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const returnValueType = Type[t.params.return_value_type];
  const fnReturnType = Type[t.params.fn_return_type];
  const enable =
  scalarTypeOf(returnValueType).kind === 'f16' || scalarTypeOf(fnReturnType).kind === 'f16' ?
  'enable f16;' :
  '';
  const code = `
${enable}

fn f() -> ${fnReturnType} {
  return ${returnValueType.create(1).wgsl()};
}
`;

  const pass = isConvertible(returnValueType, fnReturnType);
  t.expectCompileResult(pass, code);
});

const kTests = {
  no_expr: { wgsl: `return;`, pass_value: false, pass_no_value: true },
  v: { wgsl: `return v;`, pass_value: true, pass_no_value: false },
  literal: { wgsl: `return 10;`, pass_value: true, pass_no_value: false },
  expr: { wgsl: `return 1 + 2;`, pass_value: true, pass_no_value: false },
  paren_expr: { wgsl: `return (1 + 2);`, pass_value: true, pass_no_value: false },
  call: { wgsl: `return x();`, pass_value: true, pass_no_value: false },

  v_no_semicolon: { wgsl: `return v`, pass_value: false, pass_no_value: false },
  expr_no_semicolon: { wgsl: `return 1 + 2`, pass_value: false, pass_no_value: false },
  phony_assign: { wgsl: `return _ = 1;`, pass_value: false, pass_no_value: false },
  increment: { wgsl: `return v++;`, pass_value: false, pass_no_value: false },
  compound_assign: { wgsl: `return v += 4;`, pass_value: false, pass_no_value: false },
  lparen_literal: { wgsl: `return (4;`, pass_value: false, pass_no_value: false },
  literal_lparen: { wgsl: `return 4(;`, pass_value: false, pass_no_value: false },
  rparen_literal: { wgsl: `return )4;`, pass_value: false, pass_no_value: false },
  literal_rparen: { wgsl: `return 4);`, pass_value: false, pass_no_value: false },
  lparen_literal_lparen: { wgsl: `return (4(;`, pass_value: false, pass_no_value: false },
  rparen_literal_rparen: { wgsl: `return )4);`, pass_value: false, pass_no_value: false }
};

g.test('parse').
desc(`Test that 'return' statements are parsed correctly.`).
params((u) =>
u.combine('test', keysOf(kTests)).combine('fn_returns_value', [false, true])
).
fn((t) => {
  const code = `
fn f() ${t.params.fn_returns_value ? '-> i32' : ''} {
  let v = 42;
  ${kTests[t.params.test].wgsl}
}
fn x() -> i32 {
  return 1;
}
`;
  t.expectCompileResult(
    t.params.fn_returns_value ?
    kTests[t.params.test].pass_value :
    kTests[t.params.test].pass_no_value,
    code
  );
});