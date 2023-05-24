/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `Validation tests for uniformity analysis`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
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
  { op: 'workgroupBarrier', stage: 'compute' },
  { op: 'workgroupUniformLoad', stage: 'compute' },
];

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
];

function generateCondition(condition) {
  switch (condition) {
    case 'uniform_storage_ro': {
      return `ro_buffer[0] == 0`;
    }
    case 'nonuniform_storage_ro': {
      return `ro_buffer[priv_var[0]] == 0`;
    }
    case 'nonuniform_storage_rw': {
      return `rw_buffer[0] == 0`;
    }
    case 'nonuniform_builtin': {
      return `p.x == 0`;
    }
    case 'uniform_literal': {
      return `false`;
    }
    case 'uniform_const': {
      return `c`;
    }
    case 'uniform_override': {
      return `o == 0`;
    }
    case 'uniform_let': {
      return `u_let == 0`;
    }
    case 'nonuniform_let': {
      return `n_let == 0`;
    }
    case 'uniform_or': {
      return `u_let == 0 || uniform_buffer.y > 1`;
    }
    case 'nonuniform_or1': {
      return `u_let == 0 || n_let == 0`;
    }
    case 'nonuniform_or2': {
      return `n_let == 0 || u_let == 0`;
    }
    case 'uniform_and': {
      return `u_let == 0 && uniform_buffer.y > 1`;
    }
    case 'nonuniform_and1': {
      return `u_let == 0 && n_let == 0`;
    }
    case 'nonuniform_and2': {
      return `n_let == 0 && u_let == 0`;
    }
    case 'uniform_func_var': {
      return `u_f == 0`;
    }
    case 'nonuniform_func_var': {
      return `n_f == 0`;
    }
    default: {
      unreachable(`Unhandled condition`);
    }
  }
}

function generateOp(op) {
  switch (op) {
    case 'textureSample': {
      return `let x = ${op}(tex, s, vec2(0,0));\n`;
    }
    case 'textureSampleBias': {
      return `let x = ${op}(tex, s, vec2(0,0), 0);\n`;
    }
    case 'textureSampleCompare': {
      return `let x = ${op}(tex_depth, s_comp, vec2(0,0), 0);\n`;
    }
    case 'storageBarrier':
    case 'workgroupBarrier': {
      return `${op}();\n`;
    }
    case 'workgroupUniformLoad': {
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
    case 'fwidthFine': {
      return `let x = ${op}(0);\n`;
    }
    default: {
      unreachable(`Unhandled op`);
    }
  }
}

function generateConditionalStatement(statement, condition, op) {
  const code = ``;
  switch (statement) {
    case 'if': {
      return `if ${generateCondition(condition)} {
        ${generateOp(op)};
      }
      `;
    }
    case 'for': {
      return `for (; ${generateCondition(condition)};) {
        ${generateOp(op)};
      }
      `;
    }
    case 'while': {
      return `while ${generateCondition(condition)} {
        ${generateOp(op)};
      }
      `;
    }
    case 'switch': {
      return `switch u32(${generateCondition(condition)}) {
        case 0: {
          ${generateOp(op)};
        }
        default: { }
      }
      `;
    }
    default: {
      unreachable(`Unhandled statement`);
    }
  }

  return code;
}

g.test('basics')
  .desc(`Test collective operations in simple uniform or non-uniform control flow.`)
  .params(u =>
    u
      .combineWithParams(kCollectiveOps)
      .combineWithParams(kConditions)
      .combine('statement', ['if', 'for', 'while', 'switch'])
      .beginSubcases()
  )
  .fn(t => {
    let code = `
 @group(0) @binding(0) var s : sampler;
 @group(0) @binding(1) var s_comp : sampler_comparison;
 @group(0) @binding(2) var tex : texture_2d<f32>;
 @group(0) @binding(3) var tex_depth : texture_depth_2d;

 @group(1) @binding(0) var<storage, read> ro_buffer : array<f32, 4>;
 @group(1) @binding(1) var<storage, read_write> rw_buffer : array<f32, 4>;
 @group(1) @binding(2) var<uniform> uniform_buffer : vec4<f32>;

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
    type: `vec4<f32>`,
  },
  {
    builtin: `front_facing`,
    type: `bool`,
  },
  {
    builtin: `sample_index`,
    type: `u32`,
  },
  {
    builtin: `sample_mask`,
    type: `u32`,
  },
];

g.test('fragment_builtin_values')
  .desc(`Test uniformity of fragment built-in values`)
  .params(u => u.combineWithParams(kFragmentBuiltinValues).beginSubcases())
  .fn(t => {
    let cond = ``;
    switch (t.params.type) {
      case `u32`:
      case `i32`:
      case `f32`: {
        cond = `p > 0`;
        break;
      }
      case `vec4<u32>`:
      case `vec4<i32>`:
      case `vec4<f32>`: {
        cond = `p.x > 0`;
        break;
      }
      case `bool`: {
        cond = `p`;
        break;
      }
      default: {
        unreachable(`Unhandled type`);
      }
    }

    const code = `
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
    uniform: false,
  },
  {
    builtin: `local_invocation_index`,
    type: `u32`,
    uniform: false,
  },
  {
    builtin: `global_invocation_id`,
    type: `vec3<u32>`,
    uniform: false,
  },
  {
    builtin: `workgroup_id`,
    type: `vec3<u32>`,
    uniform: true,
  },
  {
    builtin: `num_workgroups`,
    type: `vec3<u32>`,
    uniform: true,
  },
];

g.test('compute_builtin_values')
  .desc(`Test uniformity of compute built-in values`)
  .params(u => u.combineWithParams(kComputeBuiltinValues).beginSubcases())
  .fn(t => {
    let cond = ``;
    switch (t.params.type) {
      case `u32`:
      case `i32`:
      case `f32`: {
        cond = `p > 0`;
        break;
      }
      case `vec3<u32>`:
      case `vec3<i32>`:
      case `vec3<f32>`: {
        cond = `p.x > 0`;
        break;
      }
      case `bool`: {
        cond = `p`;
        break;
      }
      default: {
        unreachable(`Unhandled type`);
      }
    }

    const code = `
@compute @workgroup_size(16,1,1)
fn main(@builtin(${t.params.builtin}) p : ${t.params.type}) {
  if ${cond} {
    workgroupBarrier();
  }
}
`;

    t.expectCompileResult(t.params.uniform, code);
  });
