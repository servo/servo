/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Interface matching between vertex and fragment shader validation for createRenderPipeline.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { range } from '../../../../common/util/util.js';


import { CreateRenderPipelineValidationTest } from './common.js';

function getVarName(i) {
  return `v${i}`;
}

function skipIfDisallowedInterpolationParameter(t, ...wgsl) {
  if (t.isCompatibility) {
    for (const s of wgsl) {
      if (s.includes('linear') || s.includes('sample')) {
        t.skip(`unsupported interpolation parameter in compat: ${wgsl}`);
      }
    }
  }
}

class InterStageMatchingValidationTest extends CreateRenderPipelineValidationTest {
  getVertexStateWithOutputs(outputs) {
    return {
      module: this.device.createShaderModule({
        code: `
        struct A {
            ${outputs.map((v, i) => v.replace('__', getVarName(i))).join(',\n')},
            @builtin(position) pos: vec4<f32>,
        }
        @vertex fn main() -> A {
            var vertexOut: A;
            vertexOut.pos = vec4<f32>(0.0, 0.0, 0.0, 1.0);
            return vertexOut;
        }
        `
      }),
      entryPoint: 'main'
    };
  }

  getFragmentStateWithInputs(
  inputs,
  hasBuiltinPosition = false)
  {
    return {
      targets: [{ format: 'rgba8unorm' }],
      module: this.device.createShaderModule({
        code: `
        struct B {
            ${inputs.map((v, i) => v.replace('__', getVarName(i))).join(',\n')},
            ${hasBuiltinPosition ? '@builtin(position) pos: vec4<f32>' : ''}
        }
        @fragment fn main(fragmentIn: B) -> @location(0) vec4<f32> {
            return vec4<f32>(1.0, 1.0, 1.0, 1.0);
        }
        `
      }),
      entryPoint: 'main'
    };
  }

  getDescriptorWithStates(
  vertex,
  fragment)
  {
    return {
      layout: 'auto',
      vertex,
      fragment
    };
  }
}

export const g = makeTestGroup(InterStageMatchingValidationTest);

g.test('location,mismatch').
desc(`Tests that missing declaration at the same location should fail validation.`).
params((u) =>
u.combine('isAsync', [false, true]).combineWithParams([
{ outputs: ['@location(0) __: f32'], inputs: ['@location(0) __: f32'], _success: true },
{ outputs: ['@location(0) __: f32'], inputs: ['@location(1) __: f32'], _success: false },
{ outputs: ['@location(1) __: f32'], inputs: ['@location(0) __: f32'], _success: false },
{
  outputs: ['@location(0) __: f32', '@location(1) __: f32'],
  inputs: ['@location(1) __: f32', '@location(0) __: f32'],
  _success: true
},
{
  outputs: ['@location(1) __: f32', '@location(0) __: f32'],
  inputs: ['@location(0) __: f32', '@location(1) __: f32'],
  _success: true
}]
)
).
fn((t) => {
  const { isAsync, outputs, inputs, _success } = t.params;

  const descriptor = t.getDescriptorWithStates(
    t.getVertexStateWithOutputs(outputs),
    t.getFragmentStateWithInputs(inputs)
  );

  t.doCreateRenderPipelineTest(isAsync, _success, descriptor);
});

g.test('location,superset').
desc(`Tests that validation should succeed when vertex output is superset of fragment input`).
params((u) => u.combine('isAsync', [false, true])).
fn((t) => {
  const { isAsync } = t.params;

  const descriptor = t.getDescriptorWithStates(
    t.getVertexStateWithOutputs(['@location(0) vout0: f32', '@location(1) vout1: f32']),
    t.getFragmentStateWithInputs(['@location(1) fin1: f32'])
  );

  t.doCreateRenderPipelineTest(isAsync, true, descriptor);
});

g.test('location,subset').
desc(`Tests that validation should fail when vertex output is a subset of fragment input.`).
params((u) => u.combine('isAsync', [false, true])).
fn((t) => {
  const { isAsync } = t.params;

  const descriptor = t.getDescriptorWithStates(
    t.getVertexStateWithOutputs(['@location(0) vout0: f32']),
    t.getFragmentStateWithInputs(['@location(0) fin0: f32', '@location(1) fin1: f32'])
  );

  t.doCreateRenderPipelineTest(isAsync, false, descriptor);
});

g.test('type').
desc(
  `Tests that validation should fail when type of vertex output and fragment input at the same location doesn't match.`
).
params((u) =>
u.combine('isAsync', [false, true]).combineWithParams([
{ output: 'f32', input: 'f32' },
{ output: 'i32', input: 'f32' },
{ output: 'u32', input: 'f32' },
{ output: 'u32', input: 'i32' },
{ output: 'i32', input: 'u32' },
{ output: 'vec2<f32>', input: 'vec2<f32>' },
{ output: 'vec3<f32>', input: 'vec2<f32>' },
{ output: 'vec2<f32>', input: 'vec3<f32>' },
{ output: 'vec2<f32>', input: 'f32' },
{ output: 'f32', input: 'vec2<f32>' }]
)
).
fn((t) => {
  const { isAsync, output, input } = t.params;

  const descriptor = t.getDescriptorWithStates(
    t.getVertexStateWithOutputs([`@location(0) @interpolate(flat, either) vout0: ${output}`]),
    t.getFragmentStateWithInputs([`@location(0) @interpolate(flat, either) fin0: ${input}`])
  );

  t.doCreateRenderPipelineTest(isAsync, output === input, descriptor);
});

g.test('interpolation_type').
desc(
  `Tests that validation should fail when interpolation type of vertex output and fragment input at the same location doesn't match.`
).
params((u) =>
u.combine('isAsync', [false, true]).combineWithParams([
// default is @interpolate(perspective, center)
{ output: '', input: '' },
{ output: '', input: '@interpolate(perspective)', _success: true },
{ output: '', input: '@interpolate(perspective, center)', _success: true },
{ output: '@interpolate(perspective)', input: '', _success: true },
{ output: '', input: '@interpolate(linear)' },
{ output: '@interpolate(perspective)', input: '@interpolate(perspective)' },
{ output: '@interpolate(linear)', input: '@interpolate(perspective)' },
{ output: '@interpolate(flat, either)', input: '@interpolate(perspective)' },
{ output: '@interpolate(linear)', input: '@interpolate(flat, either)' },
{
  output: '@interpolate(linear, center)',
  input: '@interpolate(linear, center)',
  _compat_success: false
}]
)
).
beforeAllSubcases((t) => {
  skipIfDisallowedInterpolationParameter(t, t.params.output, t.params.input);
}).
fn((t) => {
  const { isAsync, output, input, _success, _compat_success } = t.params;

  const descriptor = t.getDescriptorWithStates(
    t.getVertexStateWithOutputs([`@location(0) ${output} vout0: f32`]),
    t.getFragmentStateWithInputs([`@location(0) ${input} fin0: f32`])
  );

  const shouldSucceed =
  (_success ?? output === input) && (!t.isCompatibility || _compat_success !== false);

  t.doCreateRenderPipelineTest(isAsync, shouldSucceed, descriptor);
});
1;
g.test('interpolation_sampling').
desc(
  `Tests that validation should fail when interpolation sampling of vertex output and fragment input at the same location doesn't match.`
).
params((u) =>
u.combine('isAsync', [false, true]).combineWithParams([
// default is @interpolate(perspective, center)
{ output: '@interpolate(perspective)', input: '@interpolate(perspective)' },
{
  output: '@interpolate(perspective)',
  input: '@interpolate(perspective, center)',
  _success: true
},
{
  output: '@interpolate(linear, center)',
  input: '@interpolate(linear)',
  _success: true,
  _compat_success: false
},
{ output: '@interpolate(flat, either)', input: '@interpolate(flat, either)' },
{ output: '@interpolate(perspective)', input: '@interpolate(perspective, sample)' },
{ output: '@interpolate(perspective, center)', input: '@interpolate(perspective, sample)' },
{
  output: '@interpolate(perspective, center)',
  input: '@interpolate(perspective, centroid)'
},
{ output: '@interpolate(perspective, centroid)', input: '@interpolate(perspective)' }]
)
).
beforeAllSubcases((t) => {
  skipIfDisallowedInterpolationParameter(t, t.params.output, t.params.input);
}).
fn((t) => {
  const { isAsync, output, input, _success, _compat_success } = t.params;

  const descriptor = t.getDescriptorWithStates(
    t.getVertexStateWithOutputs([`@location(0) ${output} vout0: f32`]),
    t.getFragmentStateWithInputs([`@location(0) ${input} fin0: f32`])
  );

  const shouldSucceed =
  (_success ?? output === input) && (!t.isCompatibility || _compat_success !== false);

  t.doCreateRenderPipelineTest(isAsync, shouldSucceed, descriptor);
});

g.test('max_shader_variable_location').
desc(
  `Tests that validation should fail when there is location of user-defined output/input variable >= device.limits.maxInterStageShaderVariables`
).
params((u) =>
u.
combine('isAsync', [false, true])
// User defined variable location = maxInterStageShaderVariables + locationDelta
.combine('locationDelta', [0, -1, -2])
).
fn((t) => {
  const { isAsync, locationDelta } = t.params;
  const maxInterStageShaderVariables = t.device.limits.maxInterStageShaderVariables;
  const location = maxInterStageShaderVariables + locationDelta;

  const descriptor = t.getDescriptorWithStates(
    t.getVertexStateWithOutputs([`@location(${location}) vout0: f32`]),
    t.getFragmentStateWithInputs([`@location(${location}) fin0: f32`])
  );

  t.doCreateRenderPipelineTest(isAsync, location < maxInterStageShaderVariables, descriptor);
});

g.test('max_variables_count,output').
desc(
  `Tests that validation should fail when all user-defined outputs > max vertex shader output
    variables.`
).
params((u) =>
u.combine('isAsync', [false, true]).combineWithParams([
// Number of user-defined output variables in test shader =
//     device.limits.maxInterStageShaderVariables + numVariablesDelta
{ numVariablesDelta: 0, topology: 'triangle-list', _success: true },
{ numVariablesDelta: 1, topology: 'triangle-list', _success: false },
{ numVariablesDelta: 0, topology: 'point-list', _success: false },
{ numVariablesDelta: -1, topology: 'point-list', _success: true }]
)
).
fn((t) => {
  const { isAsync, numVariablesDelta, topology, _success } = t.params;

  const numVec4 = t.device.limits.maxInterStageShaderVariables + numVariablesDelta;

  const outputs = range(numVec4, (i) => `@location(${i}) vout${i}: vec4<f32>`);
  const inputs = range(numVec4, (i) => `@location(${i}) fin${i}: vec4<f32>`);

  const descriptor = t.getDescriptorWithStates(
    t.getVertexStateWithOutputs(outputs),
    t.getFragmentStateWithInputs(inputs)
  );
  descriptor.primitive = { topology };

  t.doCreateRenderPipelineTest(isAsync, _success, descriptor);
});

g.test('max_variables_count,input').
desc(
  `Tests that validation should fail when all user-defined inputs > max vertex shader output
    variables.`
).
params((u) =>
u.combine('isAsync', [false, true]).combineWithParams([
// Number of user-defined output variables in test shader =
//     device.limits.maxInterStageShaderVariables + numVariablesDelta
{ numVariablesDelta: 0, useExtraBuiltinInputs: false },
{ numVariablesDelta: 1, useExtraBuiltinInputs: false },
{ numVariablesDelta: 0, useExtraBuiltinInputs: true },
{ numVariablesDelta: -1, useExtraBuiltinInputs: true }]
)
).
fn((t) => {
  const { isAsync, numVariablesDelta, useExtraBuiltinInputs } = t.params;

  const numVec4 = t.device.limits.maxInterStageShaderVariables + numVariablesDelta;
  const numExtraVariables = useExtraBuiltinInputs ? 1 : 0;
  const numUsedVariables = numVec4 + numExtraVariables;
  const success = numUsedVariables <= t.device.limits.maxInterStageShaderVariables;

  const outputs = range(numVec4, (i) => `@location(${i}) vout${i}: vec4<f32>`);
  const inputs = range(numVec4, (i) => `@location(${i}) fin${i}: vec4<f32>`);

  if (useExtraBuiltinInputs) {
    inputs.push('@builtin(front_facing) front_facing_in: bool');
    if (!t.isCompatibility) {
      inputs.push(
        '@builtin(sample_mask) sample_mask_in: u32',
        '@builtin(sample_index) sample_index_in: u32'
      );
    }
  }

  const descriptor = t.getDescriptorWithStates(
    t.getVertexStateWithOutputs(outputs),
    t.getFragmentStateWithInputs(inputs, true)
  );

  t.doCreateRenderPipelineTest(isAsync, success, descriptor);
});