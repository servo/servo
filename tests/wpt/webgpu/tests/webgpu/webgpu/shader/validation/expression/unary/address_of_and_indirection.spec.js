/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for unary address-of and indirection (dereference)
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { keysOf } from '../../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

const kAddressSpaces = ['function', 'private', 'workgroup', 'uniform', 'storage'];
const kAccessModes = ['read', 'read_write'];
const kStorageTypes = ['bool', 'u32', 'i32', 'f32', 'f16'];
const kCompositeTypes = ['array', 'struct', 'vec', 'mat'];
const kDerefTypes = {
  deref_address_of_identifier: {
    wgsl: `(*(&a))`,
    requires_pointer_composite_access: false
  },
  deref_pointer: {
    wgsl: `(*p)`,
    requires_pointer_composite_access: false
  },
  address_of_identifier: {
    wgsl: `(&a)`,
    requires_pointer_composite_access: true
  },
  pointer: {
    wgsl: `p`,
    requires_pointer_composite_access: true
  }
};

g.test('basic').
desc(
  `Validates address-of (&) every supported variable type, ensuring the type is correct by
    assigning to an explicitly typed pointer. Also validates dereferencing the reference,
    ensuring the type is correct by assigning to an explicitly typed variable.`
).
params((u) =>
u.
combine('addressSpace', kAddressSpaces).
combine('accessMode', kAccessModes).
combine('storageType', kStorageTypes).
combine('derefType', keysOf(kDerefTypes)).
filter((t) => {
  if (t.storageType === 'bool') {
    return t.addressSpace === 'function' || t.addressSpace === 'private';
  }
  return true;
}).
filter((t) => {
  // This test does not test composite access
  return !kDerefTypes[t.derefType].requires_pointer_composite_access;
})
).
beforeAllSubcases((t) => {
  if (t.params.storageType === 'f16') {
    t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
  }
}).
fn((t) => {
  const isLocal = t.params.addressSpace === 'function';
  const deref = kDerefTypes[t.params.derefType];
  // Only specify access mode for storage buffers
  const commaAccessMode = t.params.addressSpace === 'storage' ? `, ${t.params.accessMode}` : '';

  let varDecl = '';
  if (t.params.addressSpace === 'uniform' || t.params.addressSpace === 'storage') {
    varDecl += '@group(0) @binding(0) ';
  }
  varDecl += `var<${t.params.addressSpace}${commaAccessMode}> a : VarType;`;

  const wgsl = `
      ${t.params.storageType === 'f16' ? 'enable f16;' : ''}

      alias VarType = ${t.params.storageType};
      alias PtrType = ptr<${t.params.addressSpace}, VarType ${commaAccessMode}>;

      ${isLocal ? '' : varDecl}

      fn foo() {
        ${isLocal ? varDecl : ''}
        let p : PtrType = &a;
        var deref : VarType = ${deref.wgsl};
      }
    `;

  t.expectCompileResult(true, wgsl);
});

g.test('composite').
desc(
  `Validates address-of (&) every supported variable type for composite types, ensuring the type
    is correct by assigning to an explicitly typed pointer. Also validates dereferencing the
    reference followed by member/index access, ensuring the type is correct by assigning to an
    explicitly typed variable.`
).
params((u) =>
u.
combine('addressSpace', kAddressSpaces).
combine('compositeType', kCompositeTypes).
combine('storageType', kStorageTypes).
beginSubcases().
combine('derefType', keysOf(kDerefTypes)).
combine('accessMode', kAccessModes).
filter((t) => {
  if (t.storageType === 'bool') {
    return t.addressSpace === 'function' || t.addressSpace === 'private';
  }
  return true;
}).
filter((t) => {
  if (t.compositeType === 'mat') {
    return t.storageType === 'f32' || t.storageType === 'f16';
  }
  return true;
})
).
beforeAllSubcases((t) => {
  if (t.params.storageType === 'f16') {
    t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
  }
}).
fn((t) => {
  const isLocal = t.params.addressSpace === 'function';
  const deref = kDerefTypes[t.params.derefType];
  // Only specify access mode for storage buffers
  const commaAccessMode = t.params.addressSpace === 'storage' ? `, ${t.params.accessMode}` : '';

  let varDecl = '';
  if (t.params.addressSpace === 'uniform' || t.params.addressSpace === 'storage') {
    varDecl += '@group(0) @binding(0) ';
  }
  varDecl += `var<${t.params.addressSpace}${commaAccessMode}> a : VarType;`;

  let wgsl = `
          ${t.params.storageType === 'f16' ? 'enable f16;' : ''}`;

  switch (t.params.compositeType) {
    case 'array':
      wgsl += `
          struct S { @align(16) member : ${t.params.storageType} }
          alias VarType = array<S, 10>;
          alias PtrType = ptr<${t.params.addressSpace}, VarType ${commaAccessMode}>;
          ${isLocal ? '' : varDecl}

          fn foo() {
            ${isLocal ? varDecl : ''}
            let p : PtrType = &a;
            var deref : ${t.params.storageType} = ${deref.wgsl}[0].member;
          }`;
      break;
    case 'struct':
      wgsl += `
          struct S { member : ${t.params.storageType} }
          alias VarType = S;
          alias PtrType = ptr<${t.params.addressSpace}, VarType ${commaAccessMode}>;
          ${isLocal ? '' : varDecl}

          fn foo() {
            ${isLocal ? varDecl : ''}
            let p : PtrType = &a;
            var deref : ${t.params.storageType} = ${deref.wgsl}.member;
          }`;
      break;
    case 'vec':
      wgsl += `
          alias VarType = vec3<${t.params.storageType}>;
          alias PtrType = ptr<${t.params.addressSpace}, VarType ${commaAccessMode}>;
          ${isLocal ? '' : varDecl}

          fn foo() {
            ${isLocal ? varDecl : ''}
            let p : PtrType = &a;
            var deref_member : ${t.params.storageType} = ${deref.wgsl}.x;
            var deref_index : ${t.params.storageType} = ${deref.wgsl}[0];
          }`;
      break;
    case 'mat':
      wgsl += `
          alias VarType = mat2x3<${t.params.storageType}>;
          alias PtrType = ptr<${t.params.addressSpace}, VarType ${commaAccessMode}>;
          ${isLocal ? '' : varDecl}

          fn foo() {
            ${isLocal ? varDecl : ''}
            let p : PtrType = &a;
            var deref_vec : vec3<${t.params.storageType}> = ${deref.wgsl}[0];
            var deref_elem : ${t.params.storageType} = ${deref.wgsl}[0][0];
          }`;
      break;
  }

  let shouldPass = true;
  if (
  kDerefTypes[t.params.derefType].requires_pointer_composite_access &&
  !t.hasLanguageFeature('pointer_composite_access'))
  {
    shouldPass = false;
  }

  t.expectCompileResult(shouldPass, wgsl);
});

const kInvalidCases = {
  address_of_let: `
    let a = 1;
    let p = &a;`,
  address_of_texture: `
    let p = &t;`,
  address_of_sampler: `
    let p = &s;`,
  address_of_function: `
    let p = &func;`,
  address_of_vector_elem_via_member: `
    var a : vec3<f32>();
    let p = &a.x;`,
  address_of_vector_elem_via_index: `
    var a : vec3<f32>();
    let p = &a[0];`,
  address_of_matrix_elem: `
    var a : mat2x3<f32>();
    let p = &a[0][0];`,
  deref_non_pointer: `
    var a = 1;
    let p = *a;
  `
};
g.test('invalid').
desc('Test invalid cases of unary address-of and dereference').
params((u) => u.combine('case', keysOf(kInvalidCases))).
fn((t) => {
  const wgsl = `
      @group(0) @binding(0) var s : sampler;
      @group(0) @binding(1) var t : texture_2d<f32>;
      fn func() {}
      fn main() {
        ${kInvalidCases[t.params.case]}
      }
    `;
  t.expectCompileResult(false, wgsl);
});