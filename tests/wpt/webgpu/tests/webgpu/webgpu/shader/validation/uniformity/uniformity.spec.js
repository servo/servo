/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Validation tests for uniformity analysis`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { unreachable } from '../../../../common/util/util.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

const kCollectiveOps = [
{ op: 'textureSample', stage: 'fragment' },
{ op: 'textureSampleBias', stage: 'fragment' },
{ op: 'textureSampleCompare', stage: 'fragment' },
{ op: 'dpdx', stage: 'fragment' },
{ op: 'dpdxCoarse', stage: 'fragment' },
{ op: 'dpdxFine', stage: 'fragment' },
{ op: 'dpdy', stage: 'fragment' },
{ op: 'dpdyCoarse', stage: 'fragment' },
{ op: 'dpdyFine', stage: 'fragment' },
{ op: 'fwidth', stage: 'fragment' },
{ op: 'fwidthCoarse', stage: 'fragment' },
{ op: 'fwidthFine', stage: 'fragment' },
{ op: 'storageBarrier', stage: 'compute' },
{ op: 'textureBarrier', stage: 'compute' },
{ op: 'workgroupBarrier', stage: 'compute' },
{ op: 'workgroupUniformLoad', stage: 'compute' }];


const kConditions = [
{ cond: 'uniform_storage_ro', expectation: true },
{ cond: 'nonuniform_storage_ro', expectation: false },
{ cond: 'nonuniform_storage_rw', expectation: false },
{ cond: 'nonuniform_builtin', expectation: false },
{ cond: 'uniform_literal', expectation: true },
{ cond: 'uniform_const', expectation: true },
{ cond: 'uniform_override', expectation: true },
{ cond: 'uniform_let', expectation: true },
{ cond: 'nonuniform_let', expectation: false },
{ cond: 'uniform_or', expectation: true },
{ cond: 'nonuniform_or1', expectation: false },
{ cond: 'nonuniform_or2', expectation: false },
{ cond: 'uniform_and', expectation: true },
{ cond: 'nonuniform_and1', expectation: false },
{ cond: 'nonuniform_and2', expectation: false },
{ cond: 'uniform_func_var', expectation: true },
{ cond: 'nonuniform_func_var', expectation: false },
{ cond: 'storage_texture_ro', expectation: true },
{ cond: 'storage_texture_rw', expectation: false }];


function generateCondition(condition) {
  switch (condition) {
    case 'uniform_storage_ro':{
        return `ro_buffer[0] == 0`;
      }
    case 'nonuniform_storage_ro':{
        return `ro_buffer[priv_var[0]] == 0`;
      }
    case 'nonuniform_storage_rw':{
        return `rw_buffer[0] == 0`;
      }
    case 'nonuniform_builtin':{
        return `p.x == 0`;
      }
    case 'uniform_literal':{
        return `false`;
      }
    case 'uniform_const':{
        return `c`;
      }
    case 'uniform_override':{
        return `o == 0`;
      }
    case 'uniform_let':{
        return `u_let == 0`;
      }
    case 'nonuniform_let':{
        return `n_let == 0`;
      }
    case 'uniform_or':{
        return `u_let == 0 || uniform_buffer.y > 1`;
      }
    case 'nonuniform_or1':{
        return `u_let == 0 || n_let == 0`;
      }
    case 'nonuniform_or2':{
        return `n_let == 0 || u_let == 0`;
      }
    case 'uniform_and':{
        return `u_let == 0 && uniform_buffer.y > 1`;
      }
    case 'nonuniform_and1':{
        return `u_let == 0 && n_let == 0`;
      }
    case 'nonuniform_and2':{
        return `n_let == 0 && u_let == 0`;
      }
    case 'uniform_func_var':{
        return `u_f == 0`;
      }
    case 'nonuniform_func_var':{
        return `n_f == 0`;
      }
    case 'storage_texture_ro':{
        return `textureLoad(ro_storage_texture, vec2()).x == 0`;
      }
    case 'storage_texture_rw':{
        return `textureLoad(rw_storage_texture, vec2()).x == 0`;
      }
    default:{
        unreachable(`Unhandled condition`);
      }
  }
}

function generateOp(op) {
  switch (op) {
    case 'textureSample':{
        return `let x = ${op}(tex, s, vec2(0,0));\n`;
      }
    case 'textureSampleBias':{
        return `let x = ${op}(tex, s, vec2(0,0), 0);\n`;
      }
    case 'textureSampleCompare':{
        return `let x = ${op}(tex_depth, s_comp, vec2(0,0), 0);\n`;
      }
    case 'storageBarrier':
    case 'textureBarrier':
    case 'workgroupBarrier':{
        return `${op}();\n`;
      }
    case 'workgroupUniformLoad':{
        return `let x = ${op}(&wg);`;
      }
    case 'dpdx':
    case 'dpdxCoarse':
    case 'dpdxFine':
    case 'dpdy':
    case 'dpdyCoarse':
    case 'dpdyFine':
    case 'fwidth':
    case 'fwidthCoarse':
    case 'fwidthFine':{
        return `let x = ${op}(0);\n`;
      }
    default:{
        unreachable(`Unhandled op`);
      }
  }
}

function generateConditionalStatement(statement, condition, op) {
  const code = ``;
  switch (statement) {
    case 'if':{
        return `if ${generateCondition(condition)} {
        ${generateOp(op)};
      }
      `;
      }
    case 'for':{
        return `for (; ${generateCondition(condition)};) {
        ${generateOp(op)};
      }
      `;
      }
    case 'while':{
        return `while ${generateCondition(condition)} {
        ${generateOp(op)};
      }
      `;
      }
    case 'switch':{
        return `switch u32(${generateCondition(condition)}) {
        case 0: {
          ${generateOp(op)};
        }
        default: { }
      }
      `;
      }
    default:{
        unreachable(`Unhandled statement`);
      }
  }

  return code;
}

g.test('basics').
desc(`Test collective operations in simple uniform or non-uniform control flow.`).
params((u) =>
u.
combine('statement', ['if', 'for', 'while', 'switch']).
beginSubcases().
combineWithParams(kConditions).
combineWithParams(kCollectiveOps)
).
fn((t) => {
  if (t.params.op === 'textureBarrier' || t.params.cond.startsWith('storage_texture')) {
    t.skipIfLanguageFeatureNotSupported('readonly_and_readwrite_storage_textures');
  }

  let code = `
 @group(0) @binding(0) var s : sampler;
 @group(0) @binding(1) var s_comp : sampler_comparison;
 @group(0) @binding(2) var tex : texture_2d<f32>;
 @group(0) @binding(3) var tex_depth : texture_depth_2d;

 @group(1) @binding(0) var<storage, read> ro_buffer : array<f32, 4>;
 @group(1) @binding(1) var<storage, read_write> rw_buffer : array<f32, 4>;
 @group(1) @binding(2) var<uniform> uniform_buffer : vec4<f32>;

 @group(2) @binding(0) var ro_storage_texture : texture_storage_2d<rgba8unorm, read>;
 @group(2) @binding(1) var rw_storage_texture : texture_storage_2d<rgba8unorm, read_write>;

 var<private> priv_var : array<f32, 4> = array(0,0,0,0);

 const c = false;
 override o : f32;
`;

  if (t.params.stage === 'compute') {
    code += `var<workgroup> wg : f32;\n`;
    code += ` @workgroup_size(16, 1, 1)`;
  }
  code += `@${t.params.stage}`;
  code += `\nfn main(`;
  if (t.params.stage === 'compute') {
    code += `@builtin(global_invocation_id) p : vec3<u32>`;
  } else {
    code += `@builtin(position) p : vec4<f32>`;
  }
  code += `) {
      let u_let = uniform_buffer.x;
      let n_let = rw_buffer[0];
      var u_f = uniform_buffer.z;
      var n_f = rw_buffer[1];
    `;

  // Simple control statement containing the op.
  code += generateConditionalStatement(t.params.statement, t.params.cond, t.params.op);

  code += `\n}\n`;

  t.expectCompileResult(t.params.expectation, code);
});

const kFragmentBuiltinValues = [
{
  builtin: `position`,
  type: `vec4<f32>`
},
{
  builtin: `front_facing`,
  type: `bool`
},
{
  builtin: `sample_index`,
  type: `u32`
},
{
  builtin: `sample_mask`,
  type: `u32`
},
{
  builtin: `subgroup_invocation_id`,
  type: `u32`
},
{
  builtin: `subgroup_size`,
  type: `u32`
}];


g.test('fragment_builtin_values').
desc(`Test uniformity of fragment built-in values`).
params((u) => u.combineWithParams(kFragmentBuiltinValues).beginSubcases()).
beforeAllSubcases((t) => {
  t.skipIf(
    t.isCompatibility && ['sample_index', 'sample_mask'].includes(t.params.builtin),
    'compatibility mode does not support sample_index or sample_mask'
  );
  const builtin = t.params.builtin;
  if (builtin.includes('subgroup')) {
    t.selectDeviceOrSkipTestCase('subgroups');
  }
}).
fn((t) => {
  let cond = ``;
  switch (t.params.type) {
    case `u32`:
    case `i32`:
    case `f32`:{
        cond = `p > 0`;
        break;
      }
    case `vec4<u32>`:
    case `vec4<i32>`:
    case `vec4<f32>`:{
        cond = `p.x > 0`;
        break;
      }
    case `bool`:{
        cond = `p`;
        break;
      }
    default:{
        unreachable(`Unhandled type`);
      }
  }
  const enable = t.params.builtin.includes('subgroup') ? 'enable subgroups;' : '';
  const code = `
${enable}
@group(0) @binding(0) var s : sampler;
@group(0) @binding(1) var tex : texture_2d<f32>;

@fragment
fn main(@builtin(${t.params.builtin}) p : ${t.params.type}) {
  if ${cond} {
    let texel = textureSample(tex, s, vec2<f32>(0,0));
  }
}
`;

  t.expectCompileResult(true, `diagnostic(off, derivative_uniformity);\n` + code);
  t.expectCompileResult(false, code);
});

const kComputeBuiltinValues = [
{
  builtin: `local_invocation_id`,
  type: `vec3<f32>`,
  uniform: false
},
{
  builtin: `local_invocation_index`,
  type: `u32`,
  uniform: false
},
{
  builtin: `global_invocation_id`,
  type: `vec3<u32>`,
  uniform: false
},
{
  builtin: `workgroup_id`,
  type: `vec3<u32>`,
  uniform: true
},
{
  builtin: `num_workgroups`,
  type: `vec3<u32>`,
  uniform: true
},
{
  builtin: `subgroup_invocation_id`,
  type: `u32`,
  uniform: false
},
{
  builtin: `subgroup_size`,
  type: `u32`,
  uniform: true
}];


g.test('compute_builtin_values').
desc(`Test uniformity of compute built-in values`).
params((u) => u.combineWithParams(kComputeBuiltinValues).beginSubcases()).
beforeAllSubcases((t) => {
  if (t.params.builtin.includes('subgroup')) {
    t.selectDeviceOrSkipTestCase('subgroups');
  }
}).
fn((t) => {
  let cond = ``;
  switch (t.params.type) {
    case `u32`:
    case `i32`:
    case `f32`:{
        cond = `p > 0`;
        break;
      }
    case `vec3<u32>`:
    case `vec3<i32>`:
    case `vec3<f32>`:{
        cond = `p.x > 0`;
        break;
      }
    case `bool`:{
        cond = `p`;
        break;
      }
    default:{
        unreachable(`Unhandled type`);
      }
  }
  const enable = t.params.builtin.includes('subgroup') ? 'enable subgroups;' : '';
  const code = `
${enable}
@compute @workgroup_size(16,1,1)
fn main(@builtin(${t.params.builtin}) p : ${t.params.type}) {
  if ${cond} {
    workgroupBarrier();
  }
}
`;

  t.expectCompileResult(t.params.uniform, code);
});

function generatePointerCheck(check) {
  if (check === `address`) {
    return `let tmp = workgroupUniformLoad(ptr);`;
  } else {
    // check === `contents`
    return `if test_val > 0 {
      workgroupBarrier();
    }`;
  }
}








const kPointerCases = {
  address_uniform_literal: {
    code: `let ptr = &wg_array[0];`,
    check: `address`,
    uniform: true
  },
  address_uniform_value: {
    code: `let ptr = &wg_array[uniform_value];`,
    check: `address`,
    uniform: true
  },
  address_nonuniform_value: {
    code: `let ptr = &wg_array[nonuniform_value];`,
    check: `address`,
    uniform: false
  },
  address_uniform_chain: {
    code: `let p1 = &wg_struct.x;
    let p2 = &(*p1)[uniform_value];
    let p3 = &(*p2).x;
    let ptr = &(*p3)[uniform_value];`,
    check: `address`,
    uniform: true
  },
  address_nonuniform_chain1: {
    code: `let p1 = &wg_struct.x;
    let p2 = &(*p1)[nonuniform_value];
    let p3 = &(*p2).x;
    let ptr = &(*p3)[uniform_value];`,
    check: `address`,
    uniform: false
  },
  address_nonuniform_chain2: {
    code: `let p1 = &wg_struct.x;
    let p2 = &(*p1)[uniform_value];
    let p3 = &(*p2).x;
    let ptr = &(*p3)[nonuniform_value];`,
    check: `address`,
    uniform: false
  },
  wg_uniform_load_is_uniform: {
    code: `let test_val = workgroupUniformLoad(&wg_scalar);`,
    check: `contents`,
    uniform: true
  },
  contents_scalar_uniform1: {
    code: `let ptr = &func_scalar;
    let test_val = *ptr;`,
    check: `contents`,
    uniform: true
  },
  contents_scalar_uniform2: {
    code: `func_scalar = nonuniform_value;
    let ptr = &func_scalar;
    func_scalar = 0;
    let test_val = *ptr;`,
    check: `contents`,
    uniform: true
  },
  contents_scalar_uniform3: {
    code: `let ptr = &func_scalar;
    func_scalar = nonuniform_value;
    func_scalar = uniform_value;
    let test_val = *ptr;`,
    check: `contents`,
    uniform: true
  },
  contents_scalar_nonuniform1: {
    code: `func_scalar = nonuniform_value;
    let ptr = &func_scalar;
    let test_val = *ptr;`,
    check: `contents`,
    uniform: false
  },
  contents_scalar_nonuniform2: {
    code: `let ptr = &func_scalar;
    *ptr = nonuniform_value;
    let test_val = *ptr;`,
    check: `contents`,
    uniform: false
  },
  contents_scalar_alias_uniform: {
    code: `let p = &func_scalar;
    let ptr = p;
    let test_val = *ptr;`,
    check: `contents`,
    uniform: true
  },
  contents_scalar_alias_nonuniform1: {
    code: `func_scalar = nonuniform_value;
    let p = &func_scalar;
    let ptr = p;
    let test_val = *ptr;`,
    check: `contents`,
    uniform: false
  },
  contents_scalar_alias_nonuniform2: {
    code: `let p = &func_scalar;
    *p = nonuniform_value;
    let ptr = p;
    let test_val = *ptr;`,
    check: `contents`,
    uniform: false
  },
  contents_scalar_alias_nonuniform3: {
    code: `let p = &func_scalar;
    let ptr = p;
    *p = nonuniform_value;
    let test_val = *ptr;`,
    check: `contents`,
    uniform: false
  },
  contents_scalar_alias_nonuniform4: {
    code: `let p = &func_scalar;
    func_scalar = nonuniform_value;
    let test_val = *p;`,
    check: `contents`,
    uniform: false
  },
  contents_scalar_alias_nonuniform5: {
    code: `let p = &func_scalar;
    *p = nonuniform_value;
    let test_val = func_scalar;`,
    check: `contents`,
    uniform: false
  },
  contents_array_uniform_index: {
    code: `let ptr = &func_array[uniform_value];
    let test_val = *ptr;`,
    check: `contents`,
    uniform: true
  },
  contents_array_nonuniform_index1: {
    code: `let ptr = &func_array[nonuniform_value];
    let test_val = *ptr;`,
    check: `contents`,
    uniform: false
  },
  contents_array_nonuniform_index2: {
    code: `let ptr = &func_array[lid.x];
    let test_val = *ptr;`,
    check: `contents`,
    uniform: false
  },
  contents_array_nonuniform_index3: {
    code: `let ptr = &func_array[gid.x];
    let test_val = *ptr;`,
    check: `contents`,
    uniform: false
  },
  contents_struct_uniform: {
    code: `let p1 = &func_struct.x[uniform_value].x[uniform_value].x[uniform_value];
    let test_val = *p1;`,
    check: `contents`,
    uniform: true
  },
  contents_struct_nonuniform1: {
    code: `let p1 = &func_struct.x[nonuniform_value].x[uniform_value].x[uniform_value];
    let test_val = *p1;`,
    check: `contents`,
    uniform: false
  },
  contents_struct_nonuniform2: {
    code: `let p1 = &func_struct.x[uniform_value].x[gid.x].x[uniform_value];
    let test_val = *p1;`,
    check: `contents`,
    uniform: false
  },
  contents_struct_nonuniform3: {
    code: `let p1 = &func_struct.x[uniform_value].x[uniform_value].x[lid.y];
    let test_val = *p1;`,
    check: `contents`,
    uniform: false
  },
  contents_struct_chain_uniform: {
    code: `let p1 = &func_struct.x;
    let p2 = &(*p1)[uniform_value];
    let p3 = &(*p2).x;
    let p4 = &(*p3)[uniform_value];
    let p5 = &(*p4).x;
    let p6 = &(*p5)[uniform_value];
    let test_val = *p6;`,
    check: `contents`,
    uniform: true
  },
  contents_struct_chain_nonuniform1: {
    code: `let p1 = &func_struct.x;
    let p2 = &(*p1)[nonuniform_value];
    let p3 = &(*p2).x;
    let p4 = &(*p3)[uniform_value];
    let p5 = &(*p4).x;
    let p6 = &(*p5)[uniform_value];
    let test_val = *p6;`,
    check: `contents`,
    uniform: false
  },
  contents_struct_chain_nonuniform2: {
    code: `let p1 = &func_struct.x;
    let p2 = &(*p1)[uniform_value];
    let p3 = &(*p2).x;
    let p4 = &(*p3)[gid.x];
    let p5 = &(*p4).x;
    let p6 = &(*p5)[uniform_value];
    let test_val = *p6;`,
    check: `contents`,
    uniform: false
  },
  contents_struct_chain_nonuniform3: {
    code: `let p1 = &func_struct.x;
    let p2 = &(*p1)[uniform_value];
    let p3 = &(*p2).x;
    let p4 = &(*p3)[uniform_value];
    let p5 = &(*p4).x;
    let p6 = &(*p5)[lid.y];
    let test_val = *p6;`,
    check: `contents`,
    uniform: false
  },
  contents_lhs_ref_pointer_deref1: {
    code: `*&func_scalar = uniform_value;
    let test_val = func_scalar;`,
    check: `contents`,
    uniform: true
  },
  contents_lhs_ref_pointer_deref1a: {
    code: `*&func_scalar = nonuniform_value;
    let test_val = func_scalar;`,
    check: `contents`,
    uniform: false
  },
  contents_lhs_ref_pointer_deref2: {
    code: `*&(func_array[nonuniform_value]) = uniform_value;
    let test_val = func_array[0];`,
    check: `contents`,
    uniform: false
  },
  contents_lhs_ref_pointer_deref2a: {
    code: `(func_array[nonuniform_value]) = uniform_value;
    let test_val = func_array[0];`,
    check: `contents`,
    uniform: false
  },
  contents_lhs_ref_pointer_deref3: {
    code: `*&(func_array[needs_uniform(uniform_value)]) = uniform_value;
    let test_val = func_array[0];`,
    check: `contents`,
    uniform: true
  },
  contents_lhs_ref_pointer_deref3a: {
    code: `*&(func_array[needs_uniform(nonuniform_value)]) = uniform_value;
    let test_val = func_array[0];`,
    check: `contents`,
    uniform: 'never'
  },
  contents_lhs_ref_pointer_deref4: {
    code: `*&((*&(func_struct.x[uniform_value])).x[uniform_value].x[uniform_value]) = uniform_value;
    let test_val = func_struct.x[0].x[0].x[0];`,
    check: `contents`,
    uniform: true
  },
  contents_lhs_ref_pointer_deref4a: {
    code: `*&((*&(func_struct.x[uniform_value])).x[uniform_value].x[uniform_value]) = nonuniform_value;
    let test_val = func_struct.x[0].x[0].x[0];`,
    check: `contents`,
    uniform: false
  },
  contents_lhs_ref_pointer_deref4b: {
    code: `*&((*&(func_struct.x[uniform_value])).x[uniform_value].x[nonuniform_value]) = uniform_value;
    let test_val = func_struct.x[0].x[0].x[0];`,
    check: `contents`,
    uniform: false
  },
  contents_lhs_ref_pointer_deref4c: {
    code: `*&((*&(func_struct.x[uniform_value])).x[nonuniform_value]).x[uniform_value] = uniform_value;
    let test_val = func_struct.x[0].x[0].x[0];`,
    check: `contents`,
    uniform: false
  },
  contents_lhs_ref_pointer_deref4d: {
    code: `*&((*&(func_struct.x[nonuniform_value])).x[uniform_value].x)[uniform_value] = uniform_value;
    let test_val = func_struct.x[0].x[0].x[0];`,
    check: `contents`,
    uniform: false
  },
  contents_lhs_ref_pointer_deref4e: {
    code: `*&((*&(func_struct.x[uniform_value])).x[needs_uniform(nonuniform_value)].x[uniform_value]) = uniform_value;
    let test_val = func_struct.x[0].x[0].x[0];`,
    check: `contents`,
    uniform: 'never'
  },

  // The following cases require the 'pointer_composite_access' language feature.
  contents_lhs_pointer_deref2: {
    code: `(&func_array)[uniform_value] = uniform_value;
    let test_val = func_array[0];`,
    check: `contents`,
    uniform: true,
    needs_deref_sugar: true
  },
  contents_lhs_pointer_deref2a: {
    code: `(&func_array)[nonuniform_value] = uniform_value;
    let test_val = func_array[0];`,
    check: `contents`,
    uniform: false,
    needs_deref_sugar: true
  },
  contents_lhs_pointer_deref3: {
    code: `(&func_array)[needs_uniform(uniform_value)] = uniform_value;
    let test_val = func_array[0];`,
    check: `contents`,
    uniform: true,
    needs_deref_sugar: true
  },
  contents_lhs_pointer_deref3a: {
    code: `(&func_array)[needs_uniform(nonuniform_value)] = uniform_value;
    let test_val = func_array[0];`,
    check: `contents`,
    uniform: 'never',
    needs_deref_sugar: true
  },
  contents_lhs_pointer_deref4: {
    code: `(&((&(func_struct.x[uniform_value])).x[uniform_value]).x)[uniform_value] = uniform_value;
    let test_val = func_struct.x[0].x[0].x[0];`,
    check: `contents`,
    uniform: true,
    needs_deref_sugar: true
  },
  contents_lhs_pointer_deref4a: {
    code: `(&((&(func_struct.x[uniform_value])).x[uniform_value]).x)[uniform_value] = nonuniform_value;
    let test_val = func_struct.x[0].x[0].x[0];`,
    check: `contents`,
    uniform: false,
    needs_deref_sugar: true
  },
  contents_lhs_pointer_deref4b: {
    code: `(&((&(func_struct.x[uniform_value])).x)[uniform_value]).x[nonuniform_value] = uniform_value;
    let test_val = func_struct.x[0].x[0].x[0];`,
    check: `contents`,
    uniform: false,
    needs_deref_sugar: true
  },
  contents_lhs_pointer_deref4c: {
    code: `(&((&(func_struct.x[uniform_value])).x[nonuniform_value]).x)[uniform_value] = uniform_value;
    let test_val = func_struct.x[0].x[0].x[0];`,
    check: `contents`,
    uniform: false,
    needs_deref_sugar: true
  },
  contents_lhs_pointer_deref4d: {
    code: `(&((&(func_struct.x[nonuniform_value])).x[uniform_value]).x)[uniform_value] = uniform_value;
    let test_val = func_struct.x[0].x[0].x[0];`,
    check: `contents`,
    uniform: false,
    needs_deref_sugar: true
  },
  contents_lhs_pointer_deref4e: {
    code: `(&((&(func_struct.x[uniform_value])).x)[needs_uniform(nonuniform_value)].x[uniform_value]) = uniform_value;
    let test_val = func_struct.x[0].x[0].x[0];`,
    check: `contents`,
    uniform: 'never',
    needs_deref_sugar: true
  },
  contents_rhs_pointer_deref1: {
    code: `let test_val = (&func_array)[uniform_value];`,
    check: `contents`,
    uniform: true,
    needs_deref_sugar: true
  },
  contents_rhs_pointer_deref1a: {
    code: `let test_val = (&func_array)[nonuniform_value];`,
    check: `contents`,
    uniform: false,
    needs_deref_sugar: true
  },
  contents_rhs_pointer_deref2: {
    code: `let test_val = (&func_array)[needs_uniform(nonuniform_value)];`,
    check: `contents`,
    uniform: `never`,
    needs_deref_sugar: true
  },
  contents_rhs_pointer_swizzle_uniform: {
    code: `func_vector = vec4(uniform_value);
    let test_val = dot((&func_vector).yw, vec2());`,
    check: `contents`,
    uniform: true,
    needs_deref_sugar: true
  },
  contents_rhs_pointer_swizzle_non_uniform: {
    code: `func_vector = vec4(nonuniform_value);
    let test_val = dot((&func_vector).yw, vec2());`,
    check: `contents`,
    uniform: false,
    needs_deref_sugar: true
  }
};

g.test('pointers').
desc(`Test pointer uniformity (contents and addresses)`).
params((u) => u.combine('case', keysOf(kPointerCases)).beginSubcases()).
fn((t) => {
  const testcase = kPointerCases[t.params.case];
  const code = `
var<workgroup> wg_scalar : u32;
var<workgroup> wg_array : array<u32, 16>;

struct Inner {
  x : array<u32, 4>
}
struct Middle {
  x : array<Inner, 4>
}
struct Outer {
  x : array<Middle, 4>
}
var<workgroup> wg_struct : Outer;

@group(0) @binding(0)
var<storage> uniform_value : u32;
@group(0) @binding(1)
var<storage, read_write> nonuniform_value : u32;

fn needs_uniform(val : u32) -> u32{
  if val == 0 {
    workgroupBarrier();
  }
  return val;
}

@compute @workgroup_size(16, 1, 1)
fn main(@builtin(local_invocation_id) lid : vec3<u32>,
        @builtin(global_invocation_id) gid : vec3<u32>) {
  var func_scalar : u32;
  var func_vector : vec4u;
  var func_array : array<u32, 16>;
  var func_struct : Outer;

  ${testcase.code}
`;

  const with_check =
  code +
  `
${generatePointerCheck(testcase.check)}
}`;

  if (testcase.needs_deref_sugar === true) {
    t.skipIfLanguageFeatureNotSupported('pointer_composite_access');
  }
  // Explicitly check false to distinguish from never.
  if (testcase.uniform === false) {
    const without_check = code + `}\n`;
    t.expectCompileResult(true, without_check);
  }
  t.expectCompileResult(testcase.uniform === true, with_check);
});

function expectedUniformity(uniform, init) {
  if (uniform === `always`) {
    return true;
  } else if (uniform === `init`) {
    return init === `no_init` || init === `uniform`;
  }

  // uniform == `never` (or unknown values)
  return false;
}

const kFuncVarCases = {
  no_assign: {
    typename: `u32`,
    typedecl: ``,
    assignment: ``,
    cond: `x > 0`,
    uniform: `init`
  },
  simple_uniform: {
    typename: `u32`,
    typedecl: ``,
    assignment: `x = uniform_value[0];`,
    cond: `x > 0`,
    uniform: `always`
  },
  simple_nonuniform: {
    typename: `u32`,
    typedecl: ``,
    assignment: `x = nonuniform_value[0];`,
    cond: `x > 0`,
    uniform: `never`
  },
  compound_assign_uniform: {
    typename: `u32`,
    typedecl: ``,
    assignment: `x += uniform_value[0];`,
    cond: `x > 0`,
    uniform: `init`
  },
  compound_assign_nonuniform: {
    typename: `u32`,
    typedecl: ``,
    assignment: `x -= nonuniform_value[0];`,
    cond: `x > 0`,
    uniform: `never`
  },
  unreachable_uniform: {
    typename: `u32`,
    typedecl: ``,
    assignment: `loop {
      break;
      x = uniform_value[0];
    }`,
    cond: `x > 0`,
    uniform: `init`
  },
  unreachable_nonuniform: {
    typename: `u32`,
    typedecl: ``,
    assignment: `loop {
      break;
      x = nonuniform_value[0];
    }`,
    cond: `x > 0`,
    uniform: `init`
  },
  if_no_else_uniform: {
    typename: `u32`,
    typedecl: ``,
    assignment: `if uniform_cond {
      x = uniform_value[0];
    }`,
    cond: `x > 0`,
    uniform: `init`
  },
  if_no_else_nonuniform: {
    typename: `u32`,
    typedecl: ``,
    assignment: `if uniform_cond {
      x = nonuniform_value[0];
    }`,
    cond: `x > 0`,
    uniform: `never`
  },
  if_no_then_uniform: {
    typename: `u32`,
    typedecl: ``,
    assignment: `if uniform_cond {
    } else {
      x = uniform_value[0];
    }`,
    cond: `x > 0`,
    uniform: `init`
  },
  if_no_then_nonuniform: {
    typename: `u32`,
    typedecl: ``,
    assignment: `if uniform_cond {
    } else {
      x = nonuniform_value[0];
    }`,
    cond: `x > 0`,
    uniform: `never`
  },
  if_else_uniform: {
    typename: `u32`,
    typedecl: ``,
    assignment: `if uniform_cond {
      x = uniform_value[0];
    } else {
      x = uniform_value[1];
    }`,
    cond: `x > 0`,
    uniform: `always`
  },
  if_else_nonuniform: {
    typename: `u32`,
    typedecl: ``,
    assignment: `if uniform_cond {
      x = nonuniform_value[0];
    } else {
      x = nonuniform_value[1];
    }`,
    cond: `x > 0`,
    uniform: `never`
  },
  if_else_split: {
    typename: `u32`,
    typedecl: ``,
    assignment: `if uniform_cond {
      x = uniform_value[0];
    } else {
      x = nonuniform_value[0];
    }`,
    cond: `x > 0`,
    uniform: `never`
  },
  if_unreachable_else_none: {
    typename: `u32`,
    typedecl: ``,
    assignment: `if uniform_cond {
    } else {
      return;
    }`,
    cond: `x > 0`,
    uniform: `init`
  },
  if_unreachable_else_uniform: {
    typename: `u32`,
    typedecl: ``,
    assignment: `if uniform_cond {
      x = uniform_value[0];
    } else {
      return;
    }`,
    cond: `x > 0`,
    uniform: `always`
  },
  if_unreachable_else_nonuniform: {
    typename: `u32`,
    typedecl: ``,
    assignment: `if uniform_cond {
      x = nonuniform_value[0];
    } else {
      return;
    }`,
    cond: `x > 0`,
    uniform: `never`
  },
  if_unreachable_then_none: {
    typename: `u32`,
    typedecl: ``,
    assignment: `if uniform_cond {
      return;
    }`,
    cond: `x > 0`,
    uniform: `init`
  },
  if_unreachable_then_uniform: {
    typename: `u32`,
    typedecl: ``,
    assignment: `if uniform_cond {
      return;
    } else {
      x = uniform_value[0];
    }`,
    cond: `x > 0`,
    uniform: `always`
  },
  if_unreachable_then_nonuniform: {
    typename: `u32`,
    typedecl: ``,
    assignment: `if uniform_cond {
      return;
    } else {
      x = nonuniform_value[0];
    }`,
    cond: `x > 0`,
    uniform: `never`
  },
  if_nonescaping_nonuniform: {
    typename: `u32`,
    typedecl: ``,
    assignment: `if uniform_cond {
      x = nonuniform_value[0];
      return;
    }`,
    cond: `x > 0`,
    uniform: `init`
  },
  loop_body_depends_on_continuing_uniform: {
    typename: `u32`,
    typedecl: ``,
    assignment: `loop {
      if x > 0 {
        let tmp = textureSample(t, s, vec2f(0,0));
      }
      continuing {
        x = uniform_value[0];
        break if uniform_cond;
      }
    }`,
    cond: `true`, // override the standard check
    uniform: `init`
  },
  loop_body_depends_on_continuing_nonuniform: {
    typename: `u32`,
    typedecl: ``,
    assignment: `loop {
      if x > 0 {
        let tmp = textureSample(t, s, vec2f(0,0));
      }
      continuing {
        x = nonuniform_value[0];
        break if uniform_cond;
      }
    }`,
    cond: `true`, // override the standard check
    uniform: `never`
  },
  loop_body_uniform: {
    typename: `u32`,
    typedecl: ``,
    assignment: `loop {
      x = uniform_value[0];
      continuing {
        break if uniform_cond;
      }
    }`,
    cond: `x > 0`,
    uniform: `always`
  },
  loop_body_nonuniform: {
    typename: `u32`,
    typedecl: ``,
    assignment: `loop {
      x = nonuniform_value[0];
      continuing {
        break if uniform_cond;
      }
    }`,
    cond: `x > 0`,
    uniform: `never`
  },
  loop_body_nonuniform_cond: {
    typename: `u32`,
    typedecl: ``,
    assignment: `loop {
      // The analysis doesn't recognize the content of the value.
      x = uniform_value[0];
      continuing {
        break if nonuniform_cond;
      }
    }`,
    cond: `x > 0`,
    uniform: `never`
  },
  loop_unreachable_continuing: {
    typename: `u32`,
    typedecl: ``,
    assignment: `loop {
      break;
      continuing {
        break if uniform_cond;
      }
    }`,
    cond: `x > 0`,
    uniform: `init`
  },
  loop_continuing_from_body_uniform: {
    typename: `u32`,
    typedecl: ``,
    assignment: `loop {
      x = uniform_value[0];
      continuing  {
        if x > 0 {
          let tmp = textureSample(t, s, vec2f(0,0));
        }
        break if uniform_cond;
      }
    }`,
    cond: `true`, // override the standard check
    uniform: `always`
  },
  loop_continuing_from_body_nonuniform: {
    typename: `u32`,
    typedecl: ``,
    assignment: `loop {
      x = nonuniform_value[0];
      continuing  {
        if x > 0 {
          let tmp = textureSample(t, s, vec2f(0,0));
        }
        break if uniform_cond;
      }
    }`,
    cond: `true`, // override the standard check
    uniform: `never`
  },
  loop_continuing_from_body_split1: {
    typename: `u32`,
    typedecl: ``,
    assignment: `loop {
      if uniform_cond {
        x = uniform_value[0];
      }
      continuing {
        if x > 0 {
          let tmp = textureSample(t, s, vec2f(0,0));
        }
        break if uniform_cond;
      }
    }`,
    cond: `true`, // override the standard check
    uniform: `init`
  },
  loop_continuing_from_body_split2: {
    typename: `u32`,
    typedecl: ``,
    assignment: `loop {
      if uniform_cond {
        x = nonuniform_value[0];
      }
      continuing {
        if x > 0 {
          let tmp = textureSample(t, s, vec2f(0,0));
        }
        break if uniform_cond;
      }
    }`,
    cond: `true`, // override the standard check
    uniform: `never`
  },
  loop_continuing_from_body_split3: {
    typename: `u32`,
    typedecl: ``,
    assignment: `loop {
      if uniform_cond {
        x = uniform_value[0];
      } else {
        x = uniform_value[1];
      }
      continuing {
        if x > 0 {
          let tmp = textureSample(t, s, vec2f(0,0));
        }
        break if uniform_cond;
      }
    }`,
    cond: `true`, // override the standard check
    uniform: `always`
  },
  loop_continuing_from_body_split4: {
    typename: `u32`,
    typedecl: ``,
    assignment: `loop {
      if nonuniform_cond {
        x = uniform_value[0];
      } else {
        x = uniform_value[1];
      }
      continuing {
        if x > 0 {
          let tmp = textureSample(t, s, vec2f(0,0));
        }
        break if uniform_cond;
      }
    }`,
    cond: `true`, // override the standard check
    uniform: `never`
  },
  loop_continuing_from_body_split5: {
    typename: `u32`,
    typedecl: ``,
    assignment: `loop {
      if nonuniform_cond {
        x = uniform_value[0];
      } else {
        x = uniform_value[0];
      }
      continuing {
        if x > 0 {
          let tmp = textureSample(t, s, vec2f(0,0));
        }
        break if uniform_cond;
      }
    }`,
    cond: `true`, // override the standard check
    // The analysis doesn't recognize that uniform_value[0] is assignment on all paths.
    uniform: `never`
  },
  loop_in_loop_with_continue_uniform: {
    typename: `u32`,
    typedecl: ``,
    assignment: `loop {
      loop {
        x = nonuniform_value[0];
        if nonuniform_cond {
          break;
        }
        continue;
      }
      x = uniform_value[0];
      continuing {
        if x > 0 {
          let tmp = textureSample(t, s, vec2f(0,0));
        }
        break if uniform_cond;
      }
    }`,
    cond: `true`, // override the standard check
    uniform: `always`
  },
  loop_in_loop_with_continue_nonuniform: {
    typename: `u32`,
    typedecl: ``,
    assignment: `loop {
      loop {
        x = uniform_value[0];
        if uniform_cond {
          break;
        }
        continue;
      }
      x = nonuniform_value[0];
      continuing {
        if x > 0 {
          let tmp = textureSample(t, s, vec2f(0,0));
        }
        break if uniform_cond;
      }
    }`,
    cond: `true`, // override the standard check
    uniform: `never`
  },
  after_loop_with_uniform_break_uniform: {
    typename: `u32`,
    typedecl: ``,
    assignment: `loop {
      if uniform_cond {
        x = uniform_value[0];
        break;
      }
    }`,
    cond: `x > 0`,
    uniform: `always`
  },
  after_loop_with_uniform_break_nonuniform: {
    typename: `u32`,
    typedecl: ``,
    assignment: `loop {
      if uniform_cond {
        x = nonuniform_value[0];
        break;
      }
    }`,
    cond: `x > 0`,
    uniform: `never`
  },
  after_loop_with_nonuniform_break: {
    typename: `u32`,
    typedecl: ``,
    assignment: `loop {
      if nonuniform_cond {
        x = uniform_value[0];
        break;
      }
    }`,
    cond: `x > 0`,
    uniform: `never`
  },
  after_loop_with_uniform_breaks: {
    typename: `u32`,
    typedecl: ``,
    assignment: `loop {
      if uniform_cond {
        x = uniform_value[0];
        break;
      } else {
        break;
      }
    }`,
    cond: `x > 0`,
    uniform: `init`
  },
  switch_uniform_case: {
    typename: `u32`,
    typedecl: ``,
    assignment: `switch uniform_val {
      case 0 {
        if x > 0 {
          let tmp = textureSample(t, s, vec2f(0,0));
        }
      }
      default {
      }
    }`,
    cond: `true`, // override default check
    uniform: `init`
  },
  switch_nonuniform_case: {
    typename: `u32`,
    typedecl: ``,
    assignment: `switch nonuniform_val {
      case 0 {
        if x > 0 {
          let tmp = textureSample(t, s, vec2f(0,0));
        }
      }
      default {
      }
    }`,
    cond: `true`, // override default check
    uniform: `never`
  },
  after_switch_all_uniform: {
    typename: `u32`,
    typedecl: ``,
    assignment: `switch uniform_val {
      case 0 {
        x = uniform_value[0];
      }
      case 1,2 {
        x = uniform_value[1];
      }
      default {
        x = uniform_value[2];
      }
    }`,
    cond: `x > 0`,
    uniform: `always`
  },
  after_switch_some_assign: {
    typename: `u32`,
    typedecl: ``,
    assignment: `switch uniform_val {
      case 0 {
        x = uniform_value[0];
      }
      case 1,2 {
        x = uniform_value[1];
      }
      default {
      }
    }`,
    cond: `x > 0`,
    uniform: `init`
  },
  after_switch_nonuniform: {
    typename: `u32`,
    typedecl: ``,
    assignment: `switch uniform_val {
      case 0 {
        x = uniform_value[0];
      }
      case 1,2 {
        x = uniform_value[1];
      }
      default {
        x = nonuniform_value[0];
      }
    }`,
    cond: `x > 0`,
    uniform: `never`
  },
  after_switch_with_break_nonuniform1: {
    typename: `u32`,
    typedecl: ``,
    assignment: `switch uniform_val {
      default {
        if uniform_cond {
          x = uniform_value[0];
          break;
        }
        x = nonuniform_value[0];
      }
    }`,
    cond: `x > 0`,
    uniform: `never`
  },
  after_switch_with_break_nonuniform2: {
    typename: `u32`,
    typedecl: ``,
    assignment: `switch uniform_val {
      default {
        x = uniform_value[0];
        if uniform_cond {
          x = nonuniform_value[0];
          break;
        }
      }
    }`,
    cond: `x > 0`,
    uniform: `never`
  },
  for_loop_uniform_body: {
    typename: `u32`,
    typedecl: ``,
    assignment: `for (var i = 0; i < 10; i += 1) {
      x = uniform_value[0];
    }`,
    cond: `x > 0`,
    uniform: `init`
  },
  for_loop_nonuniform_body: {
    typename: `u32`,
    typedecl: ``,
    assignment: `for (var i = 0; i < 10; i += 1) {
      x = nonuniform_value[0];
    }`,
    cond: `x > 0`,
    uniform: `never`
  },
  for_loop_uniform_body_no_condition: {
    typename: `u32`,
    typedecl: ``,
    assignment: `for (var i = 0; ; i += 1) {
      x = uniform_value[0];
      if uniform_cond {
        break;
      }
    }`,
    cond: `x > 0`,
    uniform: `always`
  },
  for_loop_nonuniform_body_no_condition: {
    typename: `u32`,
    typedecl: ``,
    assignment: `for (var i = 0; ; i += 1) {
      x = nonuniform_value[0];
      if uniform_cond {
        break;
      }
    }`,
    cond: `x > 0`,
    uniform: `never`
  },
  for_loop_uniform_increment: {
    typename: `u32`,
    typedecl: ``,
    assignment: `for (; uniform_cond; x += uniform_value[0]) {
    }`,
    cond: `x > 0`,
    uniform: `init`
  },
  for_loop_nonuniform_increment: {
    typename: `u32`,
    typedecl: ``,
    assignment: `for (; uniform_cond; x += nonuniform_value[0]) {
    }`,
    cond: `x > 0`,
    uniform: `never`
  },
  for_loop_uniform_init: {
    typename: `u32`,
    typedecl: ``,
    assignment: `for (x = uniform_value[0]; uniform_cond; ) {
    }`,
    cond: `x > 0`,
    uniform: `always`
  },
  for_loop_nonuniform_init: {
    typename: `u32`,
    typedecl: ``,
    assignment: `for (x = nonuniform_value[0]; uniform_cond;) {
    }`,
    cond: `x > 0`,
    uniform: `never`
  },
  while_loop_uniform_body: {
    typename: `u32`,
    typedecl: ``,
    assignment: `while uniform_cond {
      x = uniform_value[0];
    }`,
    cond: `x > 0`,
    uniform: `init`
  },
  while_loop_nonuniform_body: {
    typename: `u32`,
    typedecl: ``,
    assignment: `while uniform_cond {
      x = nonuniform_value[0];
    }`,
    cond: `x > 0`,
    uniform: `never`
  },
  partial_assignment_uniform: {
    typename: `block`,
    typedecl: `struct block {
      x : u32,
      y : u32
    }`,
    assignment: `x.x = uniform_value[0].x;`,
    cond: `x.x > 0`,
    uniform: `init`
  },
  partial_assignment_nonuniform: {
    typename: `block`,
    typedecl: `struct block {
      x : u32,
      y : u32
    }`,
    assignment: `x.x = nonuniform_value[0].x;`,
    cond: `x.x > 0`,
    uniform: `never`
  },
  partial_assignment_all_members_uniform: {
    typename: `block`,
    typedecl: `struct block {
      x : u32,
      y : u32
    }`,
    assignment: `x.x = uniform_value[0].x;
    x.y = uniform_value[1].y;`,
    cond: `x.x > 0`,
    uniform: `init`
  },
  partial_assignment_all_members_nonuniform: {
    typename: `block`,
    typedecl: `struct block {
      x : u32,
      y : u32
    }`,
    assignment: `x.x = nonuniform_value[0].x;
    x.y = uniform_value[0].x;`,
    cond: `x.x > 0`,
    uniform: `never`
  },
  partial_assignment_single_element_struct_uniform: {
    typename: `block`,
    typedecl: `struct block {
      x : u32
    }`,
    assignment: `x.x = uniform_value[0].x;`,
    cond: `x.x > 0`,
    uniform: `init`
  },
  partial_assignment_single_element_struct_nonuniform: {
    typename: `block`,
    typedecl: `struct block {
      x : u32
    }`,
    assignment: `x.x = nonuniform_value[0].x;`,
    cond: `x.x > 0`,
    uniform: `never`
  },
  partial_assignment_single_element_array_uniform: {
    typename: `array<u32, 1>`,
    typedecl: ``,
    assignment: `x[0] = uniform_value[0][0];`,
    cond: `x[0] > 0`,
    uniform: `init`
  },
  partial_assignment_single_element_array_nonuniform: {
    typename: `array<u32, 1>`,
    typedecl: ``,
    assignment: `x[0] = nonuniform_value[0][0];`,
    cond: `x[0] > 0`,
    uniform: `never`
  },
  nested1: {
    typename: `block`,
    typedecl: `struct block {
      x : u32,
      y : u32
    }`,
    assignment: `for (; uniform_cond; ) {
      if uniform_cond {
        x = uniform_value[0];
        break;
        x.y = nonuniform_value[0].y;
      } else {
        if uniform_cond {
          continue;
        }
        x = uniform_value[1];
      }
    }`,
    cond: `x.x > 0`,
    uniform: `init`
  },
  nested2: {
    typename: `block`,
    typedecl: `struct block {
      x : u32,
      y : u32
    }`,
    assignment: `for (; uniform_cond; ) {
      if uniform_cond {
        x = uniform_value[0];
        break;
        x.y = nonuniform_value[0].y;
      } else {
        if nonuniform_cond {
          continue;
        }
        x = uniform_value[1];
      }
    }`,
    cond: `x.x > 0`,
    uniform: `never`
  }
};

const kVarInit = {
  no_init: ``,
  uniform: `= uniform_value[3];`,
  nonuniform: `= nonuniform_value[3];`
};

g.test('function_variables').
desc(`Test uniformity of function variables`).
params((u) => u.combine('case', keysOf(kFuncVarCases)).combine('init', keysOf(kVarInit))).
fn((t) => {
  const func_case = kFuncVarCases[t.params.case];
  const code = `
${func_case.typedecl}

@group(0) @binding(0)
var<storage> uniform_value : array<${func_case.typename}, 4>;
@group(0) @binding(1)
var<storage, read_write> nonuniform_value : array<${func_case.typename}, 4>;

@group(1) @binding(0)
var t : texture_2d<f32>;
@group(1) @binding(1)
var s : sampler;

var<private> nonuniform_cond : bool = true;
const uniform_cond : bool = true;
var<private> nonuniform_val : u32 = 0;
const uniform_val : u32 = 0;

@fragment
fn main() {
  var x : ${func_case.typename} ${kVarInit[t.params.init]};

  ${func_case.assignment}

  if ${func_case.cond} {
    let tmp = textureSample(t, s, vec2f(0,0));
  }
}
`;

  const result = expectedUniformity(func_case.uniform, t.params.init);
  if (!result) {
    t.expectCompileResult(true, `diagnostic(off, derivative_uniformity);\n` + code);
  }
  t.expectCompileResult(result, code);
});

const kShortCircuitExpressionCases = {
  or_uniform_uniform: {
    code: `
      let x = uniform_cond || uniform_cond;
      if x {
        let tmp = textureSample(t, s, vec2f(0,0));
      }
    `,
    uniform: true
  },
  or_uniform_nonuniform: {
    code: `
      let x = uniform_cond || nonuniform_cond;
      if x {
        let tmp = textureSample(t, s, vec2f(0,0));
      }
    `,
    uniform: false
  },
  or_nonuniform_uniform: {
    code: `
      let x = nonuniform_cond || uniform_cond;
      if x {
        let tmp = textureSample(t, s, vec2f(0,0));
      }
    `,
    uniform: false
  },
  or_nonuniform_nonuniform: {
    code: `
      let x = nonuniform_cond || nonuniform_cond;
      if x {
        let tmp = textureSample(t, s, vec2f(0,0));
      }
    `,
    uniform: false
  },
  or_uniform_first_nonuniform: {
    code: `
      let x = textureSample(t, s, vec2f(0,0)).x == 0 || nonuniform_cond;
    `,
    uniform: true
  },
  or_uniform_second_nonuniform: {
    code: `
      let x = nonuniform_cond || textureSample(t, s, vec2f(0,0)).x == 0;
    `,
    uniform: false
  },
  and_uniform_uniform: {
    code: `
      let x = uniform_cond && uniform_cond;
      if x {
        let tmp = textureSample(t, s, vec2f(0,0));
      }
    `,
    uniform: true
  },
  and_uniform_nonuniform: {
    code: `
      let x = uniform_cond && nonuniform_cond;
      if x {
        let tmp = textureSample(t, s, vec2f(0,0));
      }
    `,
    uniform: false
  },
  and_nonuniform_uniform: {
    code: `
      let x = nonuniform_cond && uniform_cond;
      if x {
        let tmp = textureSample(t, s, vec2f(0,0));
      }
    `,
    uniform: false
  },
  and_nonuniform_nonuniform: {
    code: `
      let x = nonuniform_cond && nonuniform_cond;
      if x {
        let tmp = textureSample(t, s, vec2f(0,0));
      }
    `,
    uniform: false
  },
  and_uniform_first_nonuniform: {
    code: `
      let x = textureSample(t, s, vec2f(0,0)).x == 0 && nonuniform_cond;
    `,
    uniform: true
  },
  and_uniform_second_nonuniform: {
    code: `
      let x = nonuniform_cond && textureSample(t, s, vec2f(0,0)).x == 0;
    `,
    uniform: false
  }
};

const kPointerParamCases = {
  pointer_uniform_passthrough_value: {
    function: `fn foo(p : ptr<function, u32>) -> u32 {
      return *p;
    }`,
    call: `var x = uniform_values[0];
    let call = foo(&x);`,
    cond: `x > 0`,
    uniform: true
  },
  pointer_nonuniform_passthrough_value: {
    function: `fn foo(p : ptr<function, u32>) -> u32 {
      return *p;
    }`,
    call: `var x = uniform_values[0];
    let call = foo(&x);`,
    cond: `x > 0`,
    uniform: true
  },
  pointer_store_uniform_value: {
    function: `fn foo(p : ptr<function, u32>) {
      *p = uniform_values[0];
    }`,
    call: `var x = nonuniform_values[0];
    foo(&x);`,
    cond: `x > 0`,
    uniform: true
  },
  pointer_store_nonuniform_value: {
    function: `fn foo(p : ptr<function, u32>) {
      *p = nonuniform_values[0];
    }`,
    call: `var x = uniform_values[0];
    foo(&x);`,
    cond: `x > 0`,
    uniform: false
  },
  pointer_depends_on_nonpointer_param_uniform: {
    function: `fn foo(p : ptr<function, u32>, x : u32) {
      *p = x;
    }`,
    call: `var x = nonuniform_values[0];
    foo(&x, uniform_values[0]);`,
    cond: `x > 0`,
    uniform: true
  },
  pointer_depends_on_nonpointer_param_nonuniform: {
    function: `fn foo(p : ptr<function, u32>, x : u32) {
      *p = x;
    }`,
    call: `var x = uniform_values[0];
    foo(&x, nonuniform_values[0]);`,
    cond: `x > 0`,
    uniform: false
  },
  pointer_depends_on_pointer_param_uniform: {
    function: `fn foo(p : ptr<function, u32>, q : ptr<function, u32>) {
      *p = *q;
    }`,
    call: `var x = nonuniform_values[0];
    var y = uniform_values[0];
    foo(&x, &y);`,
    cond: `x > 0`,
    uniform: true
  },
  pointer_depends_on_pointer_param_nonuniform: {
    function: `fn foo(p : ptr<function, u32>, q : ptr<function, u32>) {
      *p = *q;
    }`,
    call: `var x = uniform_values[0];
    var y = nonuniform_values[0];
    foo(&x, &y);`,
    cond: `x > 0`,
    uniform: false
  },
  pointer_codependent1: {
    function: `fn foo(p : ptr<function, u32>, q : ptr<function, u32>) {
      if *p > 0 {
        *p = *q;
      } else {
        *q++;
      }
    }`,
    call: `var x = uniform_values[0];
    var y = uniform_values[1];
    foo(&x, &y);
    let a = x + y;`,
    cond: `a > 0`,
    uniform: true
  },
  pointer_codependent2: {
    function: `fn foo(p : ptr<function, u32>, q : ptr<function, u32>) {
      if *p > 0 {
        *p = *q;
      } else {
        *q++;
      }
    }`,
    call: `var x = uniform_values[0];
    var y = nonuniform_values[1];
    foo(&x, &y);
    let a = x + y;`,
    cond: `a > 0`,
    uniform: false
  },
  pointer_codependent3: {
    function: `fn foo(p : ptr<function, u32>, q : ptr<function, u32>) {
      if *p > 0 {
        *p = *q;
      } else {
        *q++;
      }
    }`,
    call: `var x = nonuniform_values[0];
    var y = uniform_values[1];
    foo(&x, &y);
    let a = x + y;`,
    cond: `a > 0`,
    uniform: false
  },
  pointer_codependent4: {
    function: `fn foo(p : ptr<function, u32>, q : ptr<function, u32>) {
      if *p > 0 {
        *p = *q;
      } else {
        *q++;
      }
    }`,
    call: `var x = nonuniform_values[0];
    var y = nonuniform_values[1];
    foo(&x, &y);
    let a = x + y;`,
    cond: `a > 0`,
    uniform: false
  },
  uniform_param_uniform_assignment: {
    function: `fn foo(p : ptr<function, array<u32, 2>>, idx : u32) {
      (*p)[idx] = uniform_values[0];
    }`,
    call: `var x = array(uniform_values[0], uniform_values[1]);
    foo(&x, uniform_values[3]);`,
    cond: `x[0] > 0`,
    uniform: true
  },
  uniform_param_nonuniform_assignment: {
    function: `fn foo(p : ptr<function, array<u32, 2>>, idx : u32) {
      (*p)[idx] = nonuniform_values[0];
    }`,
    call: `var x = array(uniform_values[0], uniform_values[1]);
    foo(&x, uniform_values[3]);`,
    cond: `x[0] > 0`,
    uniform: false
  },
  nonuniform_param_uniform_assignment: {
    function: `fn foo(p : ptr<function, array<u32, 2>>, idx : u32) {
      (*p)[idx] = uniform_values[0];
    }`,
    call: `var x = array(uniform_values[0], uniform_values[1]);
    foo(&x, u32(clamp(pos.x, 0, 1)));`,
    cond: `x[0] > 0`,
    uniform: false
  },
  nonuniform_param_nonuniform_assignment: {
    function: `fn foo(p : ptr<function, array<u32, 2>>, idx : u32) {
      (*p)[idx] = nonuniform_values[0];
    }`,
    call: `var x = array(uniform_values[0], uniform_values[1]);
    foo(&x, u32(clamp(pos.x, 0, 1)));`,
    cond: `x[0] > 0`,
    uniform: false
  },
  required_uniform_success: {
    function: `fn foo(p : ptr<function, u32>) {
      if *p > 0 {
        let tmp = textureSample(t,s,vec2f(0,0));
      }
    }`,
    call: `var x = uniform_values[0];
    foo(&x);`,
    cond: `uniform_cond`,
    uniform: true
  },
  required_uniform_failure: {
    function: `fn foo(p : ptr<function, u32>) {
      if *p > 0 {
        let tmp = textureSample(t,s,vec2f(0,0));
      }
    }`,
    call: `var x = nonuniform_values[0];
    foo(&x);`,
    cond: `uniform_cond`,
    uniform: false
  },
  uniform_conditional_call_assign_uniform: {
    function: `fn foo(p : ptr<function, u32>) {
      *p = uniform_values[0];
    }`,
    call: `var x = uniform_values[1];
    if uniform_cond {
      foo(&x);
    }`,
    cond: `x > 0`,
    uniform: true
  },
  uniform_conditional_call_assign_nonuniform1: {
    function: `fn foo(p : ptr<function, u32>) {
      *p = nonuniform_values[0];
    }`,
    call: `var x = uniform_values[1];
    if uniform_cond {
      foo(&x);
    }`,
    cond: `x > 0`,
    uniform: false
  },
  uniform_conditional_call_assign_nonuniform2: {
    function: `fn foo(p : ptr<function, u32>) {
      *p = uniform_values[0];
    }`,
    call: `var x = nonuniform_values[1];
    if uniform_cond {
      foo(&x);
    }`,
    cond: `x > 0`,
    uniform: false
  },
  nonuniform_conditional_call_assign_uniform: {
    function: `fn foo(p : ptr<function, u32>) {
      *p = uniform_values[0];
    }`,
    call: `var x = uniform_values[1];
    if nonuniform_cond {
      foo(&x);
    }`,
    cond: `x > 0`,
    uniform: false
  }
};

g.test('function_pointer_parameters').
desc(`Test functions and calls with pointer parameters`).
params((u) => u.combine('case', keysOf(kPointerParamCases))).
fn((t) => {
  const pointer_case = kPointerParamCases[t.params.case];
  const code = `
@group(0) @binding(0)
var t : texture_2d<f32>;
@group(0) @binding(1)
var s : sampler;

const uniform_cond = true;
var<private> nonuniform_cond = true;

@group(1) @binding(0)
var<storage> uniform_values : array<u32, 4>;
@group(1) @binding(1)
var<storage, read_write> nonuniform_values : array<u32, 4>;

${pointer_case.function}

@fragment
fn main(@builtin(position) pos : vec4f) {
  ${pointer_case.call}

  if ${pointer_case.cond} {
    let tmp = textureSample(t,s,vec2f(0,0));
  }
}
`;

  const res = pointer_case.uniform;
  if (!res) {
    t.expectCompileResult(true, `diagnostic(off, derivative_uniformity);\n` + code);
  }
  t.expectCompileResult(res, code);
});

g.test('short_circuit_expressions').
desc(`Test uniformity of expressions`).
params((u) => u.combine('case', keysOf(kShortCircuitExpressionCases))).
fn((t) => {
  const testcase = kShortCircuitExpressionCases[t.params.case];
  const code = `
@group(1) @binding(0)
var t : texture_2d<f32>;
@group(1) @binding(1)
var s : sampler;

const uniform_cond = true;
var<private> nonuniform_cond = false;

@fragment
fn main() {
  ${testcase.code}
}
`;

  const res = testcase.uniform;
  if (!res) {
    t.expectCompileResult(true, `diagnostic(off, derivative_uniformity);\n` + code);
  }
  t.expectCompileResult(res, code);
});

const kExpressionCases = {
  literal: {
    code: `1u`,
    uniform: true
  },
  uniform: {
    code: `uniform_val`,
    uniform: true
  },
  nonuniform: {
    code: `nonuniform_val`,
    uniform: false
  },
  uniform_index: {
    code: `uniform_value[uniform_val]`,
    uniform: true
  },
  nonuniform_index1: {
    code: `uniform_value[nonuniform_val]`,
    uniform: false
  },
  nonuniform_index2: {
    code: `nonuniform_value[uniform_val]`,
    uniform: false
  },
  uniform_struct: {
    code: `uniform_struct.x`,
    uniform: true
  },
  nonuniform_struct: {
    code: `nonuniform_struct.x`,
    uniform: false
  }
};

const kBinOps = {
  plus: {
    code: '+',
    test: '> 0'
  },
  minus: {
    code: '-',
    test: '> 0'
  },
  times: {
    code: '*',
    test: '> 0'
  },
  div: {
    code: '/',
    test: '> 0'
  },
  rem: {
    code: '%',
    test: '> 0'
  },
  and: {
    code: '&',
    test: '> 0'
  },
  or: {
    code: '|',
    test: '> 0'
  },
  xor: {
    code: '^',
    test: '> 0'
  },
  shl: {
    code: '<<',
    test: '> 0'
  },
  shr: {
    code: '>>',
    test: '> 0'
  },
  less: {
    code: '<',
    test: ''
  },
  lessequal: {
    code: '<=',
    test: ''
  },
  greater: {
    code: '>',
    test: ''
  },
  greaterequal: {
    code: '>=',
    test: ''
  },
  equal: {
    code: '==',
    test: ''
  },
  notequal: {
    code: '!=',
    test: ''
  }
};

g.test('binary_expressions').
desc(`Test uniformity of binary expressions`).
params((u) =>
u.
combine('e1', keysOf(kExpressionCases)).
combine('e2', keysOf(kExpressionCases)).
beginSubcases().
combine('op', keysOf(kBinOps))
).
fn((t) => {
  const e1 = kExpressionCases[t.params.e1];
  const e2 = kExpressionCases[t.params.e2];
  const op = kBinOps[t.params.op];
  const code = `
@group(0) @binding(0)
var t : texture_2d<f32>;
@group(0) @binding(1)
var s : sampler;

struct S {
  x : u32
}

const uniform_struct = S(1);
var<private> nonuniform_struct = S(1);

const uniform_value : array<u32, 2> = array(1,1);
var<private> nonuniform_value : array<u32, 2> = array(1,1);

const uniform_val : u32 = 1;
var<private> nonuniform_val : u32 = 1;

@fragment
fn main() {
  let tmp = ${e1.code} ${op.code} ${e2.code};
  if tmp ${op.test} {
    let res = textureSample(t, s, vec2f(0,0));
  }
}
`;

  const res = e1.uniform && e2.uniform;
  if (!res) {
    t.expectCompileResult(true, `diagnostic(off, derivative_uniformity);\n` + code);
  }
  t.expectCompileResult(res, code);
});

g.test('unary_expressions').
desc(`Test uniformity of uniary expressions`).
params((u) =>
u.
combine('e', keysOf(kExpressionCases)).
combine('op', ['!b_tmp', '~i_tmp > 0', '-i32(i_tmp) > 0'])
).
fn((t) => {
  const e = kExpressionCases[t.params.e];
  const code = `
@group(0) @binding(0)
var t : texture_2d<f32>;
@group(0) @binding(1)
var s : sampler;

struct S {
  x : i32
}

const uniform_struct = S(1);
var<private> nonuniform_struct = S(1);

const uniform_value : array<i32, 2> = array(1,1);
var<private> nonuniform_value : array<i32, 2> = array(1,1);

const uniform_val : i32 = 1;
var<private> nonuniform_val : i32 = 1;

@fragment
fn main() {
  let i_tmp = ${e.code};
  let b_tmp = bool(i_tmp);
  let tmp = ${t.params.op};
  if tmp {
    let res = textureSample(t, s, vec2f(0,0));
  }
}
`;

  const res = e.uniform;
  if (!res) {
    t.expectCompileResult(true, `diagnostic(off, derivative_uniformity);\n` + code);
  }
  t.expectCompileResult(res, code);
});

const kFunctionCases = {
  uniform_result: {
    function: `fn foo() -> u32 {
      return uniform_values[0];
    }`,
    call: `let call = foo();`,
    cond: `call > 0`,
    uniform: true
  },
  nonuniform_result: {
    function: `fn foo() -> u32 {
      return nonuniform_values[0];
    }`,
    call: `let call = foo();`,
    cond: `call > 0`,
    uniform: false
  },
  nonuniform_return_is_uniform_after_call: {
    function: `fn foo() {
      if nonuniform_values[0] > 0 {
        return;
      } else {
        return;
      }
    }`,
    call: `foo();`,
    cond: `uniform_cond`,
    uniform: true
  },
  uniform_passthrough_parameter: {
    function: `fn foo(x : u32) -> u32 {
      return x;
    }`,
    call: `let call = foo(uniform_values[0]);`,
    cond: `call > 0`,
    uniform: true
  },
  nonuniform_passthrough_parameter: {
    function: `fn foo(x : u32) -> u32 {
      return x;
    }`,
    call: `let call = foo(nonuniform_values[0]);`,
    cond: `call > 0`,
    uniform: false
  },
  combined_parameters1: {
    function: `fn foo(x : u32, y : u32) -> u32 {
      return x + y;
    }`,
    call: `let call = foo(uniform_values[0], uniform_values[1]);`,
    cond: `call > 0`,
    uniform: true
  },
  combined_parameters2: {
    function: `fn foo(x : u32, y : u32) -> u32 {
      return x + y;
    }`,
    call: `let call = foo(nonuniform_values[0], uniform_values[1]);`,
    cond: `call > 0`,
    uniform: false
  },
  combined_parameters3: {
    function: `fn foo(x : u32, y : u32) -> u32 {
      return x + y;
    }`,
    call: `let call = foo(uniform_values[0], nonuniform_values[1]);`,
    cond: `call > 0`,
    uniform: false
  },
  combined_parameters4: {
    function: `fn foo(x : u32, y : u32) -> u32 {
      return x + y;
    }`,
    call: `let call = foo(nonuniform_values[0], nonuniform_values[1]);`,
    cond: `call > 0`,
    uniform: false
  },
  uniform_parameter_cf_after_nonuniform_expr: {
    function: `fn foo(x : bool, y : vec4f) -> f32 {
      return select(0, y.x, x);
    }`,
    call: `let call = foo(nonuniform_cond || uniform_cond, textureSample(t,s,vec2f(0,0)));`,
    cond: `uniform_cond`,
    uniform: true
  },
  required_uniform_function_call_in_uniform_cf: {
    function: `fn foo() -> vec4f {
      return textureSample(t,s,vec2f(0,0));
    }`,
    call: `if uniform_cond {
      let call = foo();
    }`,
    cond: `uniform_cond`,
    uniform: true
  },
  required_uniform_function_call_in_nonuniform_cf: {
    function: `fn foo() -> vec4f {
      return textureSample(t,s,vec2f(0,0));
    }`,
    call: `if nonuniform_cond {
      let call = foo();
    }`,
    cond: `uniform_cond`,
    uniform: false
  },
  required_uniform_function_call_in_nonuniform_cf2: {
    function: `@diagnostic(warning, derivative_uniformity)
    fn foo() -> vec4f {
      return textureSample(t,s,vec2f(0,0));
    }`,
    call: `if nonuniform_cond {
      let call = foo();
      let sample = textureSample(t,s,vec2f(0,0));
    }`,
    cond: `uniform_cond`,
    uniform: false
  },
  required_uniform_function_call_depends_on_uniform_param: {
    function: `fn foo(x : bool) -> vec4f {
      if x {
        return textureSample(t,s,vec2f(0,0));
      }
      return vec4f(0);
    }`,
    call: `let call = foo(uniform_cond);`,
    cond: `uniform_cond`,
    uniform: true
  },
  required_uniform_function_call_depends_on_nonuniform_param: {
    function: `fn foo(x : bool) -> vec4f {
      if x {
        return textureSample(t,s,vec2f(0,0));
      }
      return vec4f(0);
    }`,
    call: `let call = foo(nonuniform_cond);`,
    cond: `uniform_cond`,
    uniform: false
  },
  dpdx_nonuniform_result: {
    function: ``,
    call: `let call = dpdx(1);`,
    cond: `call > 0`,
    uniform: false
  },
  dpdy_nonuniform_result: {
    function: ``,
    call: `let call = dpdy(1);`,
    cond: `call > 0`,
    uniform: false
  },
  dpdxCoarse_nonuniform_result: {
    function: ``,
    call: `let call = dpdxCoarse(1);`,
    cond: `call > 0`,
    uniform: false
  },
  dpdyCoarse_nonuniform_result: {
    function: ``,
    call: `let call = dpdyCoarse(1);`,
    cond: `call > 0`,
    uniform: false
  },
  dpdxFine_nonuniform_result: {
    function: ``,
    call: `let call = dpdxFine(1);`,
    cond: `call > 0`,
    uniform: false
  },
  dpdyFine_nonuniform_result: {
    function: ``,
    call: `let call = dpdyFine(1);`,
    cond: `call > 0`,
    uniform: false
  },
  fwidth_nonuniform_result: {
    function: ``,
    call: `let call = fwidth(1);`,
    cond: `call > 0`,
    uniform: false
  },
  fwidthCoarse_nonuniform_result: {
    function: ``,
    call: `let call = fwidthCoarse(1);`,
    cond: `call > 0`,
    uniform: false
  },
  fwidthFine_nonuniform_result: {
    function: ``,
    call: `let call = fwidthFine(1);`,
    cond: `call > 0`,
    uniform: false
  },
  textureSample_nonuniform_result: {
    function: ``,
    call: `let call = textureSample(t,s,vec2f(0,0));`,
    cond: `call.x > 0`,
    uniform: false
  },
  textureSampleBias_nonuniform_result: {
    function: ``,
    call: `let call = textureSampleBias(t,s,vec2f(0,0), 0);`,
    cond: `call.x > 0`,
    uniform: false
  },
  textureSampleCompare_nonuniform_result: {
    function: ``,
    call: `let call = textureSampleCompare(td,sd,vec2f(0,0), 0);`,
    cond: `call > 0`,
    uniform: false
  },
  textureDimensions_uniform_input_uniform_result: {
    function: ``,
    call: `let call = textureDimensions(t);`,
    cond: `call.x > 0`,
    uniform: true
  },
  textureGather_uniform_input_uniform_result: {
    function: ``,
    call: `let call = textureGather(0,t,s,vec2f(0,0));`,
    cond: `call.x > 0`,
    uniform: true
  },
  textureGatherCompare_uniform_input_uniform_result: {
    function: ``,
    call: `let call = textureGatherCompare(td,sd,vec2f(0,0), 0);`,
    cond: `call.x > 0`,
    uniform: true
  },
  textureLoad_uniform_input_uniform_result: {
    function: ``,
    call: `let call = textureLoad(t,vec2u(0,0),0);`,
    cond: `call.x > 0`,
    uniform: true
  },
  textureNumLayers_uniform_input_uniform_result: {
    function: ``,
    call: `let call = textureNumLayers(ta);`,
    cond: `call > 0`,
    uniform: true
  },
  textureNumLevels_uniform_input_uniform_result: {
    function: ``,
    call: `let call = textureNumLevels(t);`,
    cond: `call > 0`,
    uniform: true
  },
  textureNumSamples_uniform_input_uniform_result: {
    function: ``,
    call: `let call = textureNumSamples(ts);`,
    cond: `call > 0`,
    uniform: true
  },
  textureSampleLevel_uniform_input_uniform_result: {
    function: ``,
    call: `let call = textureSampleLevel(t,s,vec2f(0,0),0);`,
    cond: `call.x > 0`,
    uniform: true
  },
  textureSampleGrad_uniform_input_uniform_result: {
    function: ``,
    call: `let call = textureSampleGrad(t,s,vec2f(0,0),vec2f(0,0),vec2f(0,0));`,
    cond: `call.x > 0`,
    uniform: true
  },
  textureSampleCompareLevel_uniform_input_uniform_result: {
    function: ``,
    call: `let call = textureSampleCompareLevel(td,sd,vec2f(0,0), 0);`,
    cond: `call > 0`,
    uniform: true
  },
  textureSampleBaseClampToEdge_uniform_input_uniform_result: {
    function: ``,
    call: `let call = textureSampleBaseClampToEdge(t,s,vec2f(0,0));`,
    cond: `call.x > 0`,
    uniform: true
  },
  min_uniform_input_uniform_result: {
    function: ``,
    call: `let call = min(0,0);`,
    cond: `call > 0`,
    uniform: true
  },
  value_constructor_uniform_input_uniform_result: {
    function: ``,
    call: `let call = vec2u(0,0);`,
    cond: `call.x > 0`,
    uniform: true
  }
};

g.test('functions').
desc(`Test uniformity of function calls (non-pointer parameters)`).
params((u) => u.combine('case', keysOf(kFunctionCases))).
fn((t) => {
  const func_case = kFunctionCases[t.params.case];
  const code = `
@group(0) @binding(0)
var t : texture_2d<f32>;
@group(0) @binding(1)
var s : sampler;
@group(0) @binding(2)
var td : texture_depth_2d;
@group(0) @binding(3)
var sd : sampler_comparison;
@group(0) @binding(4)
var ta : texture_2d_array<f32>;
@group(0) @binding(5)
var ts : texture_multisampled_2d<f32>;

const uniform_cond = true;
var<private> nonuniform_cond = true;

@group(1) @binding(0)
var<storage> uniform_values : array<u32, 4>;
@group(1) @binding(1)
var<storage, read_write> nonuniform_values : array<u32, 4>;

${func_case.function}

@fragment
fn main() {
  ${func_case.call}

  if ${func_case.cond} {
    let tmp = textureSample(t,s,vec2f(0,0));
  }
}
`;

  const res = func_case.uniform;
  if (!res) {
    t.expectCompileResult(true, `diagnostic(off, derivative_uniformity);\n` + code);
  }
  t.expectCompileResult(res, code);
});