/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = 'Test pointer type validation';import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { kAccessModeInfo, kAddressSpaceInfo } from '../../types.js';
import {
  pointerType,
  explicitSpaceExpander,
  accessModeExpander,
  getVarDeclShader,
  supportsWrite } from

'../decl/util.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

g.test('missing_type').
desc('Test that pointer types require an element type').
params((u) =>
u.
combine('aspace', ['function', 'private', 'workgroup', 'storage', 'uniform']).
combine('comma', ['', ','])
).
fn((t) => {
  const code = `alias T = ptr<${t.params.aspace}${t.params.comma}>;`;
  t.expectCompileResult(false, code);
});

g.test('address_space').
desc('Test address spaces in pointer type parameterization').
params((u) =>
u.
combine('aspace', [
'function',
'private',
'workgroup',
'storage',
'uniform',
'handle',
'bad_aspace']
).
combine('comma', ['', ','])
).
fn((t) => {
  const code = `alias T = ptr<${t.params.aspace}, u32${t.params.comma}>;`;
  const success = t.params.aspace !== 'handle' && t.params.aspace !== 'bad_aspace';
  t.expectCompileResult(success, code);
});

g.test('access_mode').
desc('Test access mode in pointer type parameterization').
params((u) =>
u.
combine('aspace', ['function', 'private', 'storage', 'uniform', 'workgroup']).
combine('access', ['read', 'write', 'read_write']).
combine('comma', ['', ','])
).
fn((t) => {
  // Default access mode is tested above.
  const code = `alias T = ptr<${t.params.aspace}, u32, ${t.params.access}${t.params.comma}>;`;
  const success = t.params.aspace === 'storage' && t.params.access !== 'write';
  t.expectCompileResult(success, code);
});








const kTypeCases = {
  // Scalars
  bool: { type: `bool`, storable: true, aspace: 'function' },
  u32: { type: `u32`, storable: true },
  i32: { type: `i32`, storable: true },
  f32: { type: `f32`, storable: true },
  f16: { type: `f16`, storable: true, f16: true },

  // Vectors
  vec2u: { type: `vec2u`, storable: true },
  vec3i: { type: `vec3i`, storable: true },
  vec4f: { type: `vec4f`, storable: true },
  vec2_bool: { type: `vec2<bool>`, storable: true, aspace: 'workgroup' },
  vec3h: { type: `vec3h`, storable: true, f16: true },

  // Matrices
  mat2x2f: { type: `mat2x2f`, storable: true },
  mat3x4h: { type: `mat3x4h`, storable: true, f16: true },

  // Atomics
  atomic_u32: { type: `atomic<u32>`, storable: true },
  atomic_i32: { type: `atomic<i32>`, storable: true },

  // Arrays
  array_sized_u32: { type: `array<u32, 4>`, storable: true },
  array_sized_vec4f: { type: `array<vec4f, 16>`, storable: true },
  array_sized_S: { type: `array<S, 2>`, storable: true },
  array_runtime_u32: { type: `array<u32>`, storable: true },
  array_runtime_S: { type: `array<S>`, storable: true },
  array_runtime_atomic_u32: { type: `array<atomic<u32>>`, storable: true },
  array_override_u32: { type: `array<u32, o>`, storable: true, aspace: 'workgroup' },

  // Structs
  struct_S: { type: `S`, storable: true },
  struct_T: { type: `T`, storable: true },

  // Pointers
  ptr_function_u32: { type: `ptr<function, u32>`, storable: false },
  ptr_workgroup_bool: { type: `ptr<workgroup, bool>`, storable: false },

  // Sampler (while storable, can only be in the handle address space)
  sampler: { type: `sampler`, storable: false },

  // Texture (while storable, can only be in the handle address space)
  texture_2d: { type: `texture_2d<f32>`, storable: false },

  // Alias
  alias: { type: `u32_alias`, storable: true },

  // Reference
  reference: { type: `ref<function, u32>`, storable: false, aspace: 'function' }
};

g.test('type').
desc('Tests that pointee type must be storable').
params((u) => u.combine('case', keysOf(kTypeCases))).
beforeAllSubcases((t) => {
  if (kTypeCases[t.params.case].f16) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const testcase = kTypeCases[t.params.case];
  const aspace = testcase.aspace ?? 'storage';
  const access = testcase.type.includes('atomic') ? ', read_write' : '';
  const code = `${testcase.f16 ? 'enable f16;' : ''}
    override o : u32;
    struct S { x : u32 }
    struct T { s : array<S> }
    alias u32_alias = u32;
    alias Type = ptr<${aspace}, ${testcase.type}${access}>;`;
  t.expectCompileResult(testcase.storable, code);
});

// Address spaces that can hold an i32 variable.
const kNonHandleAddressSpaces = keysOf(kAddressSpaceInfo).filter(
  (as) => as !== 'handle'
);

g.test('let_ptr_explicit_type_matches_var').
desc(
  'Let-declared pointer with explicit type initialized from var with same address space and access mode'
).
specURL('https://w3.org/TR#ref-ptr-types').
params((u) =>
u // Generate non-handle variables in all valid permutations of address space and access mode.
.combine('addressSpace', kNonHandleAddressSpaces).
expand('explicitSpace', explicitSpaceExpander).
combine('explicitAccess', [false, true]).
expand('accessMode', accessModeExpander).
combine('stage', ['compute']) // Only need to check compute shaders
// Vary the store type.
.combine('ptrStoreType', ['i32', 'u32'])
).
fn((t) => {
  // Match the address space and access mode.
  const prog = getVarDeclShader(t.params, `let p: ${pointerType(t.params)} = &x;`);
  const ok = t.params.ptrStoreType === 'i32'; // The store type matches the variable's store type.

  t.expectCompileResult(ok, prog);
});

g.test('let_ptr_reads').
desc('Validate reading via ptr when permitted by access mode').
params((u) =>
u // Generate non-handle variables in all valid permutations of address space and access mode.
.combine('addressSpace', kNonHandleAddressSpaces).
expand('explicitSpace', explicitSpaceExpander).
combine('explicitAccess', [false, true]).
expand('accessMode', accessModeExpander).
combine('stage', ['compute']) // Only need to check compute shaders
.combine('inferPtrType', [false, true]).
combine('ptrStoreType', ['i32'])
).
fn((t) => {
  // Try reading through the pointer.
  const typePart = t.params.inferPtrType ? `: ${pointerType(t.params)}` : '';
  const prog = getVarDeclShader(t.params, `let p${typePart} = &x; let read = *p;`);
  const ok = true; // We can always read.

  t.expectCompileResult(ok, prog);
});

g.test('let_ptr_writes').
desc('Validate writing via ptr when permitted by access mode').
specURL('https://w3.org/TR#ref-ptr-types').
params((u) =>
u // Generate non-handle variables in all valid permutations of address space and access mode.
.combine('addressSpace', kNonHandleAddressSpaces).
expand('explicitSpace', explicitSpaceExpander).
combine('explicitAccess', [false, true]).
expand('accessMode', accessModeExpander).
combine('stage', ['compute']) // Only need to check compute shaders
.combine('inferPtrType', [false, true]).
combine('ptrStoreType', ['i32'])
).
fn((t) => {
  // Try writing through the pointer.
  const typePart = t.params.inferPtrType ? `: ${pointerType(t.params)}` : '';
  const prog = getVarDeclShader(t.params, `let p${typePart} = &x; *p = 42;`);
  const ok = supportsWrite(t.params);

  t.expectCompileResult(ok, prog);
});

g.test('ptr_handle_space_invalid').fn((t) => {
  t.expectCompileResult(false, 'alias p = ptr<handle,u32>;');
});

g.test('ptr_bad_store_type').
params((u) => u.combine('storeType', ['undeclared', 'clamp', '1'])).
fn((t) => {
  t.expectCompileResult(false, `alias p = ptr<private,${t.params.storeType}>;`);
});

g.test('ptr_address_space_never_uses_access_mode').
params((u) =>
u.
combine(
  'addressSpace',
  keysOf(kAddressSpaceInfo).filter((i) => kAddressSpaceInfo[i].spellAccessMode === 'never')
).
combine('accessMode', keysOf(kAccessModeInfo))
).
fn((t) => {
  const prog = `alias pty = ptr<${t.params.addressSpace},u32,;${t.params.accessMode}>;`;
  t.expectCompileResult(false, prog);
});

const kStoreTypeNotInstantiable = {
  ptr: 'alias p = ptr<storage,ptr<private,i32>>;',
  privateAtomic: 'alias p = ptr<private,atomic<u32>>;',
  functionAtomic: 'alias p = ptr<function,atomic<u32>>;',
  uniformAtomic: 'alias p = ptr<uniform,atomic<u32>>;',
  workgroupRTArray: 'alias p = ptr<workgroup,array<i32>>;',
  uniformRTArray: 'alias p = ptr<uniform,array<i32>>;',
  privateRTArray: 'alias p = ptr<private,array<i32>>;',
  functionRTArray: 'alias p = ptr<function,array<i32>>;',
  RTArrayNotLast: 'struct S { a: array<i32>, b: i32 } alias p = ptr<storage,S>;',
  nestedRTArray: 'struct S { a: array<i32>, b: i32 } struct { s: S } alias p = ptr<storage,T>;'
};

g.test('ptr_not_instantiable').
desc(
  'Validate that ptr type must correspond to a variable that could be declared somewhere; test bad cases'
).
params((u) => u.combine('case', keysOf(kStoreTypeNotInstantiable))).
fn((t) => {
  t.expectCompileResult(false, kStoreTypeNotInstantiable[t.params.case]);
});