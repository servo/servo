/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Validation tests for attributes`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

const kPossibleValues = {
  val: '32',
  expr: '30 + 2',
  override: 'a_override',
  user_func: 'a_func()',
  const_func: 'min(4, 8)',
  const: 'a_const'
};
const kAttributeUsage = {
  align: '@align($val)',
  binding: '@binding($val) @group(0)',
  group: '@binding(1) @group($val)',
  id: '@id($val)',
  location: '@location($val)',
  size: '@size($val)',
  workgroup_size: '@workgroup_size($val, $val, $val)'
};
const kAllowedUsages = {
  align: ['val', 'expr', 'const', 'const_func'],
  binding: ['val', 'expr', 'const', 'const_func'],
  group: ['val', 'expr', 'const', 'const_func'],
  id: ['val', 'expr', 'const', 'const_func'],
  location: ['val', 'expr', 'const', 'const_func'],
  size: ['val', 'expr', 'const', 'const_func'],
  workgroup_size: ['val', 'expr', 'const', 'const_func', 'override']
};

g.test('expressions').
desc(`Tests attributes which allow expressions`).
params((u) =>
u.combine('value', keysOf(kPossibleValues)).combine('attribute', keysOf(kAllowedUsages))
).
fn((t) => {
  const attributes = {
    align: '',
    binding: '@binding(0) @group(0)',
    group: '@binding(1) @group(1)',
    id: '@id(2)',
    location: '@location(0)',
    size: '',
    workgroup_size: '@workgroup_size(1)'
  };

  const val = kPossibleValues[t.params.value];
  attributes[t.params.attribute] = kAttributeUsage[t.params.attribute].replace(/(\$val)/g, val);

  const code = `
fn a_func() -> i32 {
    return 4;
}

const a_const = -2 + 10;
override a_override: i32 = 2;

${attributes.id} override my_id: i32 = 4;

struct B {
  ${attributes.align} ${attributes.size} a: i32,
}

${attributes.binding}
var<uniform> uniform_buffer_1: B;

${attributes.group}
var<uniform> uniform_buffer_2: B;

@fragment
fn main() -> ${attributes.location} vec4<f32> {
  return vec4<f32>(.4, .2, .3, .1);
}

@compute
${attributes.workgroup_size}
fn compute_main() {}
`;

  const pass = kAllowedUsages[t.params.attribute].includes(t.params.value);
  t.expectCompileResult(pass, code);
});