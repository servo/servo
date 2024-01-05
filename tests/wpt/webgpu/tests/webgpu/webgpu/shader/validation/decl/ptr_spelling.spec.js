/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validate spelling of pointer types.

Pointer types may appear.

They are parameterized by:
- address space, always
- store type
- and access mode, as specified by the table in Address Spaces.
   Concretely, only 'storage' address space allows it, and allows 'read', and 'read_write'.

A pointer type can be spelled only if it corresponds to a variable that could be
declared in the program.  So we need to test combinations against possible variable
declarations.
`; // This file tests spelling of the pointer type on let-declared pointers.
//
// Spelling of pointer-typed parameters on user-declared functions is tested by
// webgpu:shader,validation,functions,restrictions:function_parameter_types:"*"

import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { kAccessModeInfo, kAddressSpaceInfo } from '../../types.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

import {
  pointerType,
  explicitSpaceExpander,
  accessModeExpander,
  getVarDeclShader,
  supportsWrite } from

'./util.js';

// Address spaces that can hold an i32 variable.
const kNonHandleAddressSpaces = keysOf(kAddressSpaceInfo).filter(
  (as) => as !== 'handle'
);

export const g = makeTestGroup(ShaderValidationTest);

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