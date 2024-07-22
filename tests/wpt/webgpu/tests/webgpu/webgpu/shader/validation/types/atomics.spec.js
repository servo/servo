/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for atomic types

Tests covered:
* Base type
* Address spaces
* Invalid operations (non-exhaustive)

Note: valid operations (e.g. atomic built-in functions) are tested in the builtin tests.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

g.test('type').
desc('Test of the underlying atomic data type').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#atomic-types').
params((u) =>
u.combine('type', [
'u32',
'i32',
'f32',
'f16',
'bool',
'vec2u',
'vec3i',
'vec4f',
'mat2x2f',
'R',
'S',
'array<u32, 1>',
'array<i32, 4>',
'array<u32>',
'array<i32>',
'atomic<u32>',
'atomic<i32>',
'sampler']
)
).
beforeAllSubcases((t) => {
  if (t.params.type === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const code = `
struct S {
  x : u32
}
struct T {
  x : i32
}
struct R {
  x : f32
}

struct Test {
  x : atomic<${t.params.type}>
}
`;

  const expect = t.params.type === 'u32' || t.params.type === 'i32';
  t.expectCompileResult(expect, code);
});

const kSpecifierCases = {
  no_type: {
    code: `alias T = atomic;`,
    valid: false
  },
  missing_l_template: {
    code: `alias T = atomici32>;`,
    valid: false
  },
  missing_r_template: {
    code: `alias T = atomic<i32;`,
    valid: false
  },
  template_comma: {
    code: `alias T = atomic<i32,>;`,
    valid: true
  },
  missing_template_param: {
    code: `alias T = atomic<>;`,
    valid: false
  },
  space_in_specifier: {
    code: `alias T = atomic <i32>;`,
    valid: true
  },
  space_as_l_template: {
    code: `alias T = atomic i32>;`,
    valid: false
  },
  comment: {
    code: `alias T = atomic
    /* comment */
    <i32>;`,
    valid: true
  }
};

g.test('parse').
desc('Test atomic parsing').
params((u) => u.combine('case', keysOf(kSpecifierCases))).
fn((t) => {
  const testcase = kSpecifierCases[t.params.case];
  t.expectCompileResult(testcase.valid, testcase.code);
});

g.test('address_space').
desc('Test allowed address spaces for atomics').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#atomic-types').
params((u) =>
u.
combine('aspace', [
'storage',
'workgroup',
'storage-ro',
'uniform',
'private',
'function',
'function-let']
).
beginSubcases().
combine('type', ['i32', 'u32'])
).
fn((t) => {
  let moduleVar = ``;
  let functionVar = '';
  switch (t.params.aspace) {
    case 'storage-ro':
      moduleVar = `@group(0) @binding(0) var<storage> x : atomic<${t.params.type}>;\n`;
      break;
    case 'storage':
      moduleVar = `@group(0) @binding(0) var<storage, read_write> x : atomic<${t.params.type}>;\n`;
      break;
    case 'uniform':
      moduleVar = `@group(0) @binding(0) var<uniform> x : atomic<${t.params.type}>;\n`;
      break;
    case 'workgroup':
    case 'private':
      moduleVar = `var<${t.params.aspace}> x : atomic<${t.params.type}>;\n`;
      break;
    case 'function':
      functionVar = `var x : atomic<${t.params.type}>;\n`;
      break;
    case 'function-let':
      functionVar = `let x : atomic<${t.params.type}>;\n`;
      break;
  }
  const code = `
${moduleVar}

fn foo() {
  ${functionVar}
}
`;

  const expect = t.params.aspace === 'storage' || t.params.aspace === 'workgroup';
  t.expectCompileResult(expect, code);
});

const kInvalidOperations = {
  add: `a1 + a2`,
  load: `a1`,
  store: `a1 = 1u`,
  deref: `*a1 = 1u`,
  equality: `a1 == a2`,
  abs: `abs(a1)`,
  address_abs: `abs(&a1)`
};

g.test('invalid_operations').
desc('Tests that a selection of invalid operations are invalid').
params((u) => u.combine('op', keysOf(kInvalidOperations))).
fn((t) => {
  const code = `
var<workgroup> a1 : atomic<u32>;
var<workgroup> a2 : atomic<u32>;

fn foo() {
  let x : u32 = ${kInvalidOperations[t.params.op]};
}
`;

  t.expectCompileResult(false, code);
});

g.test('trailing_comma').
desc('Test that trailing commas are accepted').
params((u) => u.combine('type', ['u32', 'i32']).combine('comma', ['', ','])).
fn((t) => {
  const code = `alias T = atomic<${t.params.type}${t.params.comma}>;`;
  t.expectCompileResult(true, code);
});