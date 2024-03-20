/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for host-shareable types.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

// The set of types and their properties.
const kTypes = {
  // Scalars.
  bool: {
    isHostShareable: false,
    isConstructible: true,
    isFixedFootprint: true,
    requiresF16: false
  },
  i32: {
    isHostShareable: true,
    isConstructible: true,
    isFixedFootprint: true,
    requiresF16: false
  },
  u32: {
    isHostShareable: true,
    isConstructible: true,
    isFixedFootprint: true,
    requiresF16: false
  },
  f32: {
    isHostShareable: true,
    isConstructible: true,
    isFixedFootprint: true,
    requiresF16: false
  },
  f16: {
    isHostShareable: true,
    isConstructible: true,
    isFixedFootprint: true,
    requiresF16: true
  },

  // Vectors.
  'vec2<bool>': {
    isHostShareable: false,
    isConstructible: true,
    isFixedFootprint: true,
    requiresF16: false
  },
  vec3i: {
    isHostShareable: true,
    isConstructible: true,
    isFixedFootprint: true,
    requiresF16: false
  },
  vec4u: {
    isHostShareable: true,
    isConstructible: true,
    isFixedFootprint: true,
    requiresF16: false
  },
  vec2f: {
    isHostShareable: true,
    isConstructible: true,
    isFixedFootprint: true,
    requiresF16: false
  },
  vec3h: {
    isHostShareable: true,
    isConstructible: true,
    isFixedFootprint: true,
    requiresF16: true
  },

  // Matrices.
  mat2x2f: {
    isHostShareable: true,
    isConstructible: true,
    isFixedFootprint: true,
    requiresF16: false
  },
  mat3x4h: {
    isHostShareable: true,
    isConstructible: true,
    isFixedFootprint: true,
    requiresF16: true
  },

  // Atomics.
  'atomic<i32>': {
    isHostShareable: true,
    isConstructible: false,
    isFixedFootprint: true,
    requiresF16: false
  },
  'atomic<u32>': {
    isHostShareable: true,
    isConstructible: false,
    isFixedFootprint: true,
    requiresF16: false
  },

  // Arrays.
  'array<vec4<bool>>': {
    isHostShareable: false,
    isConstructible: false,
    isFixedFootprint: false,
    requiresF16: false
  },
  'array<vec4<bool>, 4>': {
    isHostShareable: false,
    isConstructible: true,
    isFixedFootprint: true,
    requiresF16: false
  },
  'array<vec4u>': {
    isHostShareable: true,
    isConstructible: false,
    isFixedFootprint: false,
    requiresF16: false
  },
  'array<vec4u, 4>': {
    isHostShareable: true,
    isConstructible: true,
    isFixedFootprint: true,
    requiresF16: false
  },
  'array<vec4u, array_size_const>': {
    isHostShareable: true,
    isConstructible: true,
    isFixedFootprint: true,
    requiresF16: false
  },
  'array<vec4u, array_size_override>': {
    isHostShareable: false,
    isConstructible: false,
    isFixedFootprint: true,
    requiresF16: false
  },

  // Structures.
  S_u32: {
    isHostShareable: true,
    isConstructible: true,
    isFixedFootprint: true,
    requiresF16: false
  },
  S_bool: {
    isHostShareable: false,
    isConstructible: true,
    isFixedFootprint: true,
    requiresF16: false
  },
  S_S_bool: {
    isHostShareable: false,
    isConstructible: true,
    isFixedFootprint: true,
    requiresF16: false
  },
  S_array_vec4u: {
    isHostShareable: true,
    isConstructible: false,
    isFixedFootprint: false,
    requiresF16: false
  },
  S_array_vec4u_4: {
    isHostShareable: true,
    isConstructible: true,
    isFixedFootprint: true,
    requiresF16: false
  },
  S_array_bool_4: {
    isHostShareable: false,
    isConstructible: true,
    isFixedFootprint: true,
    requiresF16: false
  },

  // Misc.
  'ptr<function, u32>': {
    isHostShareable: false,
    isConstructible: false,
    isFixedFootprint: false,
    requiresF16: false
  },
  sampler: {
    isHostShareable: false,
    isConstructible: false,
    isFixedFootprint: false,
    requiresF16: false
  },
  'texture_2d<f32>': {
    isHostShareable: false,
    isConstructible: false,
    isFixedFootprint: false,
    requiresF16: false
  }
};

g.test('module_scope_types').
desc('Test that only types that are allowed for a given address space are accepted.').
params((u) =>
u.
combine('type', keysOf(kTypes)).
combine('kind', [
'comment',
'handle',
'private',
'storage_ro',
'storage_rw',
'uniform',
'workgroup']
).
combine('via_alias', [false, true])
).
beforeAllSubcases((t) => {
  if (kTypes[t.params.type].requiresF16) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const type = kTypes[t.params.type];
  const isAtomic = t.params.type.indexOf('atomic') > -1;

  let decl = '<>';
  let shouldPass = false;
  switch (t.params.kind) {
    case 'comment':
      // Control case to make sure all types are spelled correctly.
      // We always emit an alias to the target type.
      decl = '// ';
      shouldPass = true;
      break;
    case 'handle':
      decl = '@group(0) @binding(0) var foo : ';
      shouldPass = t.params.type.indexOf('texture') > -1 || t.params.type.indexOf('sampler') > -1;
      break;
    case 'private':
      decl = 'var<private> foo : ';
      shouldPass = type.isConstructible;
      break;
    case 'storage_ro':
      decl = '@group(0) @binding(0) var<storage, read> foo : ';
      shouldPass = type.isHostShareable && !isAtomic;
      break;
    case 'storage_rw':
      decl = '@group(0) @binding(0) var<storage, read_write> foo : ';
      shouldPass = type.isHostShareable;
      break;
    case 'uniform':
      decl = '@group(0) @binding(0) var<uniform> foo : ';
      shouldPass = type.isHostShareable && type.isConstructible;
      break;
    case 'workgroup':
      decl = 'var<workgroup> foo : ';
      shouldPass = type.isFixedFootprint;
      break;
  }

  const wgsl = `${type.requiresF16 ? 'enable f16;' : ''}
    const array_size_const = 4;
    override array_size_override = 4;

    struct S_u32 { a : u32 }
    struct S_bool { a : bool }
    struct S_S_bool { a : S_bool }
    struct S_array_vec4u { a : array<u32> }
    struct S_array_vec4u_4 { a : array<vec4u, 4> }
    struct S_array_bool_4 { a : array<bool, 4> }

    alias MyType = ${t.params.type};

    ${decl} ${t.params.via_alias ? 'MyType' : t.params.type};
    `;

  t.expectCompileResult(shouldPass, wgsl);
});

g.test('function_scope_types').
desc('Test that only types that are allowed for a given address space are accepted.').
params((u) =>
u.
combine('type', keysOf(kTypes)).
combine('kind', ['comment', 'var']).
combine('via_alias', [false, true])
).
beforeAllSubcases((t) => {
  if (kTypes[t.params.type].requiresF16) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const type = kTypes[t.params.type];

  let decl = '<>';
  let shouldPass = false;
  switch (t.params.kind) {
    case 'comment':
      // Control case to make sure all types are spelled correctly.
      // We always emit an alias to the target type.
      decl = '// ';
      shouldPass = true;
      break;
    case 'var':
      decl = 'var foo : ';
      shouldPass = type.isConstructible;
      break;
  }

  const wgsl = `${type.requiresF16 ? 'enable f16;' : ''}
    const array_size_const = 4;
    override array_size_override = 4;

    struct S_u32 { a : u32 }
    struct S_bool { a : bool }
    struct S_S_bool { a : S_bool }
    struct S_array_vec4u { a : array<u32> }
    struct S_array_vec4u_4 { a : array<vec4u, 4> }
    struct S_array_bool_4 { a : array<bool, 4> }

    alias MyType = ${t.params.type};

    fn foo() {
      ${decl} ${t.params.via_alias ? 'MyType' : t.params.type};
    }`;

  t.expectCompileResult(shouldPass, wgsl);
});

g.test('module_scope_initializers').
desc('Test that initializers are only supported on address spaces that allow them.').
params((u) =>
u.
combine('initializer', [false, true]).
combine('kind', ['private', 'storage_ro', 'storage_rw', 'uniform', 'workgroup'])
).
fn((t) => {
  let decl = '<>';
  switch (t.params.kind) {
    case 'private':
      decl = 'var<private> foo : ';
      break;
    case 'storage_ro':
      decl = '@group(0) @binding(0) var<storage, read> foo : ';
      break;
    case 'storage_rw':
      decl = '@group(0) @binding(0) var<storage, read_write> foo : ';
      break;
    case 'uniform':
      decl = '@group(0) @binding(0) var<uniform> foo : ';
      break;
    case 'workgroup':
      decl = 'var<workgroup> foo : ';
      break;
  }

  const wgsl = `${decl} u32${t.params.initializer ? ' = 42u' : ''};`;
  t.expectCompileResult(t.params.kind === 'private' || !t.params.initializer, wgsl);
});

g.test('handle_initializer').
desc('Test that initializers are not allowed for handle types').
params((u) =>
u.combine('initializer', [false, true]).combine('type', ['sampler', 'texture_2d<f32>'])
).
fn((t) => {
  const wgsl = `
    @group(0) @binding(0) var foo : ${t.params.type};
    @group(0) @binding(1) var bar : ${t.params.type}${t.params.initializer ? ' = foo' : ''};`;
  t.expectCompileResult(!t.params.initializer, wgsl);
});

// A list of u32 initializers and their validity for the private address space.
const kInitializers = {
  'u32()': true,
  '42u': true,
  'u32(sqrt(42.0))': true,
  'user_func()': false,
  my_const_42u: true,
  my_override_42u: true,
  another_private_var: false,
  'vec4u(1, 2, 3, 4)[my_const_42u / 20]': true,
  'vec4u(1, 2, 3, 4)[my_override_42u / 20]': true,
  'vec4u(1, 2, 3, 4)[another_private_var / 20]': false
};

g.test('initializer_kind').
desc(
  'Test that initializers must be const-expression or override-expression for the private address space.'
).
params((u) =>
u.combine('initializer', keysOf(kInitializers)).combine('addrspace', ['private', 'function'])
).
fn((t) => {
  let wgsl = `
    const my_const_42u = 42u;
    override my_override_42u : u32;
    var<private> another_private_var = 42u;

    fn user_func() -> u32 {
      return 42u;
    }
    `;

  if (t.params.addrspace === 'private') {
    wgsl += `
      var<private> foo : u32 = ${t.params.initializer};`;
  } else {
    wgsl += `
      fn foo() {
        var bar : u32 = ${t.params.initializer};
      }`;
  }
  t.expectCompileResult(
    t.params.addrspace === 'function' || kInitializers[t.params.initializer],
    wgsl
  );
});

g.test('function_addrspace_at_module_scope').
desc('Test that the function address space is not allowed at module scope.').
params((u) => u.combine('addrspace', ['private', 'function'])).
fn((t) => {
  t.expectCompileResult(
    t.params.addrspace === 'private',
    `var<${t.params.addrspace}> foo : i32;`
  );
});

// A list of resource variable declarations.
const kResourceDecls = {
  uniform: 'var<uniform> buffer : vec4f;',
  storage: 'var<storage> buffer : vec4f;',
  texture: 'var t : texture_2d<f32>;',
  sampler: 'var s : sampler;'
};

g.test('binding_point_on_resources').
desc('Test that resource variables must have both @group and @binding attributes.').
params((u) =>
u.
combine('decl', keysOf(kResourceDecls)).
combine('group', ['', '@group(0)']).
combine('binding', ['', '@binding(0)'])
).
fn((t) => {
  const shouldPass = t.params.group !== '' && t.params.binding !== '';
  const wgsl = `${t.params.group} ${t.params.binding} ${kResourceDecls[t.params.decl]}`;
  t.expectCompileResult(shouldPass, wgsl);
});

g.test('binding_point_on_non_resources').
desc('Test that non-resource variables cannot have either @group or @binding attributes.').
params((u) =>
u.
combine('addrspace', ['private', 'workgroup']).
combine('group', ['', '@group(0)']).
combine('binding', ['', '@binding(0)'])
).
fn((t) => {
  const shouldPass = t.params.group === '' && t.params.binding === '';
  const wgsl = `${t.params.group} ${t.params.binding} var<${t.params.addrspace}> foo : i32;`;
  t.expectCompileResult(shouldPass, wgsl);
});

g.test('binding_point_on_function_var').
desc('Test that function variables cannot have either @group or @binding attributes.').
params((u) => u.combine('group', ['', '@group(0)']).combine('binding', ['', '@binding(0)'])).
fn((t) => {
  const shouldPass = t.params.group === '' && t.params.binding === '';
  const wgsl = `
    fn foo() {
      ${t.params.group} ${t.params.binding} var bar : i32;
    }`;
  t.expectCompileResult(shouldPass, wgsl);
});

g.test('binding_collisions').
desc('Test that binding points can collide iff they are not used by the same entry point.').
params((u) =>
u.
combine('a_group', [0, 1]).
combine('b_group', [0, 1]).
combine('a_binding', [0, 1]).
combine('b_binding', [0, 1]).
combine('b_use', ['same', 'different'])
).
fn((t) => {
  const wgsl = `
    @group(${t.params.a_group}) @binding(${t.params.a_binding}) var<uniform> a : vec4f;
    @group(${t.params.b_group}) @binding(${t.params.b_binding}) var<uniform> b : vec4f;

    @fragment
    fn main1() {
      _ = a;
      ${
  t.params.b_use === 'same' ?
  '' :
  `
      }

    @fragment
    fn main2() {`
  }
      _ = b;
    }`;

  const collision =
  t.params.a_group === t.params.b_group && t.params.a_binding === t.params.b_binding;
  const shouldFail = collision && t.params.b_use === 'same';
  t.expectCompileResult(!shouldFail, wgsl);
});

g.test('binding_collision_unused_helper').
desc('Test that binding points can collide in an unused helper function.').
fn((t) => {
  const wgsl = `
    @group(0) @binding(0) var<uniform> a : vec4f;
    @group(0) @binding(0) var<uniform> b : vec4f;

    fn foo() {
      _ = a;
      _ = b;
    }`;

  t.expectCompileResult(true, wgsl);
});