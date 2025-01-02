/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for host-shareable types.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { kAccessModeInfo, kAddressSpaceInfo } from '../../types.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

import {
  explicitSpaceExpander,
  getVarDeclShader,
  accessModeExpander,
  supportsRead,
  supportsWrite } from

'./util.js';

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

g.test('address_space_access_mode').
desc('Test that only storage accepts an access mode').
params((u) =>
u.
combine('address_space', ['private', 'storage', 'uniform', 'function', 'workgroup']).
combine('access_mode', ['', 'read', 'write', 'read_write']).
combine('trailing_comma', [true, false])
).
fn((t) => {
  let fdecl = ``;
  let mdecl = ``;
  // Most address spaces do not accept an access mode, but should accept no
  // template argument or a trailing comma.
  let shouldPass = t.params.access_mode === '';
  let suffix = ``;
  if (t.params.access_mode === '') {
    suffix += t.params.trailing_comma ? ',' : '';
  } else {
    suffix += `,${t.params.access_mode}`;
    suffix += t.params.trailing_comma ? ',' : '';
  }
  // 'handle' unchecked since it is untypable.
  switch (t.params.address_space) {
    case 'private':
      mdecl = `var<private${suffix}> x : u32;`;
      break;
    case 'storage':
      mdecl = `@group(0) @binding(0) var<storage${suffix}> x : u32;`;
      shouldPass = t.params.access_mode !== 'write';
      break;
    case 'uniform':
      mdecl = `@group(0) @binding(0) var<uniform${suffix}> x : u32;`;
      break;
    case 'workgroup':
      mdecl = `var<workgroup${suffix}> x : u32;`;
      break;
    case 'function':
      fdecl = `var<function${suffix}> x : u32;`;
      break;
  }
  const code = `${mdecl}
    fn foo() {
      ${fdecl}
    }`;
  t.expectCompileResult(shouldPass, code);
});

// Address spaces that can hold an i32 variable.
const kNonHandleAddressSpaces = keysOf(kAddressSpaceInfo).filter(
  (as) => as !== 'handle'
);

g.test('explicit_access_mode').
desc('Validate uses of an explicit access mode on a var declaration').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#var-decls').
params(
  (u) =>
  u.
  combine('addressSpace', kNonHandleAddressSpaces).
  combine('explicitSpace', [true, false])
  // Only keep cases where:
  //   *if* the address space must be specified on a var decl (e.g. var<private>)
  //   then the address space will actually be specified in this test case.
  .filter((t) => kAddressSpaceInfo[t.addressSpace].spell !== 'must' || t.explicitSpace).
  combine('explicitAccess', [true]).
  combine('accessMode', keysOf(kAccessModeInfo)).
  combine('stage', ['compute']) // Only need to check compute shaders
).
fn((t) => {
  const prog = getVarDeclShader(t.params);
  const info = kAddressSpaceInfo[t.params.addressSpace];

  const ok =
  // The address space must be explicitly specified.
  t.params.explicitSpace &&
  // The address space must allow an access mode to be spelled, and the
  // access mode must be in the list of modes for the address space.
  info.spellAccessMode !== 'never' &&
  info.accessModes.includes(t.params.accessMode);

  t.expectCompileResult(ok, prog);
});

g.test('implicit_access_mode').
desc('Validate an implicit access mode on a var declaration').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#var-decls').
params(
  (u) =>
  u.
  combine('addressSpace', kNonHandleAddressSpaces).
  expand('explicitSpace', explicitSpaceExpander).
  combine('explicitAccess', [false]).
  combine('accessMode', ['']).
  combine('stage', ['compute']) // Only need to check compute shaders
).
fn((t) => {
  const prog = getVarDeclShader(t.params);

  // 7.3 var Declarations
  // "The access mode always has a default value,.."
  const ok = true;

  t.expectCompileResult(ok, prog);
});

g.test('read_access').
desc('A variable can be read from when the access mode permits').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#var-decls').
params(
  (u) =>
  u.
  combine('addressSpace', kNonHandleAddressSpaces).
  expand('explicitSpace', explicitSpaceExpander).
  combine('explicitAccess', [false, true]).
  expand('accessMode', accessModeExpander).
  combine('stage', ['compute']) // Only need to check compute shaders
).
fn((t) => {
  const prog = getVarDeclShader(t.params, 'let copy = x;');
  const ok = supportsRead(t.params);
  t.expectCompileResult(ok, prog);
});

g.test('write_access').
desc('A variable can be written to when the access mode permits').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#var-decls').
params(
  (u) =>
  u.
  combine('addressSpace', kNonHandleAddressSpaces).
  expand('explicitSpace', explicitSpaceExpander).
  combine('explicitAccess', [false, true]).
  expand('accessMode', accessModeExpander).
  combine('stage', ['compute']) // Only need to check compute shaders
).
fn((t) => {
  const prog = getVarDeclShader(t.params, 'x = 0;');
  const ok = supportsWrite(t.params);
  t.expectCompileResult(ok, prog);
});

const kTestTypes = [
'f32',
'i32',
'u32',
'bool',
'vec2<f32>',
'vec2<i32>',
'vec2<u32>',
'vec2<bool>',
'vec3<f32>',
'vec3<i32>',
'vec3<u32>',
'vec3<bool>',
'vec4<f32>',
'vec4<i32>',
'vec4<u32>',
'vec4<bool>',
'mat2x2<f32>',
'mat2x3<f32>',
'mat2x4<f32>',
'mat3x2<f32>',
'mat3x3<f32>',
'mat3x4<f32>',
'mat4x2<f32>',
'mat4x3<f32>',
'mat4x4<f32>',
// [1]: 12 is a random number here. find a solution to replace it.
'array<f32, 12>',
'array<i32, 12>',
'array<u32, 12>',
'array<bool, 12>'];


g.test('initializer_type').
desc(
  `
  If present, the initializer's type must match the store type of the variable.
  Testing scalars, vectors, and matrices of every dimension and type.
  TODO: add test for: structs - arrays of vectors and matrices - arrays of different length
`
).
params((u) => u.beginSubcases().combine('lhsType', kTestTypes).combine('rhsType', kTestTypes)).
fn((t) => {
  const { lhsType, rhsType } = t.params;

  const code = `
      @fragment
      fn main() {
        var a : ${lhsType} = ${rhsType}();
      }
    `;

  const expectation = lhsType === rhsType;
  t.expectCompileResult(expectation, code);
});

g.test('var_access_mode_bad_other_template_contents').
desc(
  'A variable declaration with explicit access mode with varying other template list contents'
).
specURL('https://gpuweb.github.io/gpuweb/wgsl/#var-decls').
params((u) =>
u.
combine('accessMode', ['read', 'read_write']).
combine('prefix', ['storage,', '', ',']).
combine('suffix', [',storage', ',read', ',', ''])
).
fn((t) => {
  const prog = `@group(0) @binding(0)
                  var<${t.params.prefix}${t.params.accessMode}${t.params.suffix}> x: i32;`;
  const ok =
  t.params.prefix === 'storage,' && (t.params.suffix === '' || t.params.suffix === ',');
  t.expectCompileResult(ok, prog);
});

g.test('var_access_mode_bad_template_delim').
desc('A variable declaration has explicit access mode with varying template list delimiters').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#var-decls').
params((u) =>
u.
combine('accessMode', ['read', 'read_write']).
combine('prefix', ['', '<', '>', ',']).
combine('suffix', ['', '<', '>', ','])
).
fn((t) => {
  const prog = `@group(0) @binding(0)
                  var ${t.params.prefix}storage,${t.params.accessMode}${t.params.suffix} x: i32;`;
  const ok = t.params.prefix === '<' && t.params.suffix === '>';
  t.expectCompileResult(ok, prog);
});

g.test('shader_stage').
desc('Test the limitations of address space and shader stage').
params((u) =>
u.
combine('stage', ['compute', 'vertex', 'fragment']).
combine('kind', [
'handle_ro',
'handle_wo',
'handle_rw',
'function',
'private',
'storage_ro',
'storage_rw',
'uniform',
'workgroup']
)
).
fn((t) => {
  t.skipIf(
    !t.hasLanguageFeature('readonly_and_readwrite_storage_textures') &&
    t.params.kind === 'handle_rw',
    'Unsupported language feature'
  );
  let mdecl = ``;
  let fdecl = ``;
  let expect = true;
  switch (t.params.kind) {
    case 'handle_ro':
      mdecl = `@group(0) @binding(0) var v : sampler;`;
      break;
    case 'handle_wo':
      mdecl = `@group(0) @binding(0) var v : texture_storage_2d<r32uint, write>;`;
      expect = t.params.stage !== 'vertex';
      break;
    case 'handle_rw':
      mdecl = `@group(0) @binding(0) var v : texture_storage_2d<r32uint, read_write>;`;
      expect = t.params.stage !== 'vertex';
      break;
    case 'function':
      fdecl = `var v : u32;`;
      break;
    case 'private':
      mdecl = `var<private> v : i32;`;
      break;
    case 'storage_ro':
      mdecl = `@group(0) @binding(0) var<storage> v : u32;`;
      break;
    case 'storage_rw':
      mdecl = `@group(0) @binding(0) var<storage, read_write> v : u32;`;
      expect = t.params.stage !== 'vertex';
      break;
    case 'uniform':
      mdecl = `@group(0) @binding(0) var<uniform> v : u32;`;
      break;
    case 'workgroup':
      mdecl = `var<workgroup> v : u32;`;
      expect = t.params.stage === 'compute';
      break;
  }
  let func = ``;
  switch (t.params.stage) {
    case 'compute':
      func = `@compute @workgroup_size(1)
        fn main() {
          ${fdecl}
          _ = v;
        }`;
      break;
    case 'vertex':
      func = `@vertex
        fn main() -> @builtin(position) vec4f {
          ${fdecl}
          _ = v;
          return vec4f();
        }`;
      break;
    case 'fragment':
      func = `@fragment
        fn main() {
          ${fdecl}
          _ = v;
        }`;
      break;
  }

  const code = `
    ${mdecl}
    ${func}`;
  t.expectCompileResult(expect, code);
});