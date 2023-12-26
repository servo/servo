/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
This tests entry point validation of compute/render pipelines and their shader modules.

The entryPoint in shader module include standard "main" and others.
The entryPoint assigned in descriptor include:
- Undefined with matching entry point for stage
- Matching case (control case)
- Empty string
- Mistyping
- Containing invalid char, including space and control codes (Null character)
- Unicode entrypoints and their ASCIIfied version

TODO:
- Fine-tune test cases to reduce number by removing trivially similar cases
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { kDefaultVertexShaderCode, getShaderWithEntryPoint } from '../../../util/shader.js';
import { ValidationTest } from '../validation_test.js';

export const g = makeTestGroup(ValidationTest);

const kEntryPointTestCases = [
{ shaderModuleEntryPoint: 'main', stageEntryPoint: 'main' },
{ shaderModuleEntryPoint: 'main', stageEntryPoint: '' },
{ shaderModuleEntryPoint: 'main', stageEntryPoint: 'main\0' },
{ shaderModuleEntryPoint: 'main', stageEntryPoint: 'main\0a' },
{ shaderModuleEntryPoint: 'main', stageEntryPoint: 'mian' },
{ shaderModuleEntryPoint: 'main', stageEntryPoint: 'main ' },
{ shaderModuleEntryPoint: 'main', stageEntryPoint: 'ma in' },
{ shaderModuleEntryPoint: 'main', stageEntryPoint: 'main\n' },
{ shaderModuleEntryPoint: 'mian', stageEntryPoint: 'mian' },
{ shaderModuleEntryPoint: 'mian', stageEntryPoint: 'main' },
{ shaderModuleEntryPoint: 'mainmain', stageEntryPoint: 'mainmain' },
{ shaderModuleEntryPoint: 'mainmain', stageEntryPoint: 'foo' },
{ shaderModuleEntryPoint: 'main_t12V3', stageEntryPoint: 'main_t12V3' },
{ shaderModuleEntryPoint: 'main_t12V3', stageEntryPoint: 'main_t12V5' },
{ shaderModuleEntryPoint: 'main_t12V3', stageEntryPoint: '_main_t12V3' },
{ shaderModuleEntryPoint: 'séquençage', stageEntryPoint: 'séquençage' },
{ shaderModuleEntryPoint: 'séquençage', stageEntryPoint: 'séquençage' }];


g.test('compute').
desc(
  `
Tests calling createComputePipeline(Async) with valid compute stage shader and different entryPoints,
and check that the APIs only accept matching entryPoint.
`
).
params((u) =>
u.
combine('isAsync', [true, false]).
combine('shaderModuleStage', ['compute', 'vertex', 'fragment']).
beginSubcases().
combine('provideEntryPoint', [true, false]).
combine('extraEntryPoint', [true, false]).
combineWithParams(kEntryPointTestCases)
).
fn((t) => {
  const {
    isAsync,
    provideEntryPoint,
    extraEntryPoint,
    shaderModuleStage,
    shaderModuleEntryPoint,
    stageEntryPoint
  } = t.params;
  const entryPoint = provideEntryPoint ? stageEntryPoint : undefined;
  let code = getShaderWithEntryPoint(shaderModuleStage, shaderModuleEntryPoint);
  if (extraEntryPoint) {
    code += ` ${getShaderWithEntryPoint(shaderModuleStage, 'extra')}`;
  }
  const descriptor = {
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({
        code
      }),
      entryPoint
    }
  };
  let _success = true;
  if (shaderModuleStage !== 'compute') {
    _success = false;
  }
  if (!provideEntryPoint && extraEntryPoint) {
    _success = false;
  }
  if (shaderModuleEntryPoint !== stageEntryPoint && provideEntryPoint) {
    _success = false;
  }
  t.doCreateComputePipelineTest(isAsync, _success, descriptor);
});

g.test('vertex').
desc(
  `
Tests calling createRenderPipeline(Async) with valid vertex stage shader and different entryPoints,
and check that the APIs only accept matching entryPoint.
`
).
params((u) =>
u.
combine('isAsync', [true, false]).
combine('shaderModuleStage', ['compute', 'vertex', 'fragment']).
beginSubcases().
combine('provideEntryPoint', [true, false]).
combine('extraEntryPoint', [true, false]).
combineWithParams(kEntryPointTestCases)
).
fn((t) => {
  const {
    isAsync,
    provideEntryPoint,
    extraEntryPoint,
    shaderModuleStage,
    shaderModuleEntryPoint,
    stageEntryPoint
  } = t.params;
  const entryPoint = provideEntryPoint ? stageEntryPoint : undefined;
  let code = getShaderWithEntryPoint(shaderModuleStage, shaderModuleEntryPoint);
  if (extraEntryPoint) {
    code += ` ${getShaderWithEntryPoint(shaderModuleStage, 'extra')}`;
  }
  const descriptor = {
    layout: 'auto',
    vertex: {
      module: t.device.createShaderModule({ code }),
      entryPoint
    }
  };
  let _success = true;
  if (shaderModuleStage !== 'vertex') {
    _success = false;
  }
  if (!provideEntryPoint && extraEntryPoint) {
    _success = false;
  }
  if (shaderModuleEntryPoint !== stageEntryPoint && provideEntryPoint) {
    _success = false;
  }
  t.doCreateRenderPipelineTest(isAsync, _success, descriptor);
});

g.test('fragment').
desc(
  `
Tests calling createRenderPipeline(Async) with valid fragment stage shader and different entryPoints,
and check that the APIs only accept matching entryPoint.
`
).
params((u) =>
u.
combine('isAsync', [true, false]).
combine('shaderModuleStage', ['compute', 'vertex', 'fragment']).
beginSubcases().
combine('provideEntryPoint', [true, false]).
combine('extraEntryPoint', [true, false]).
combineWithParams(kEntryPointTestCases)
).
fn((t) => {
  const {
    isAsync,
    provideEntryPoint,
    extraEntryPoint,
    shaderModuleStage,
    shaderModuleEntryPoint,
    stageEntryPoint
  } = t.params;
  const entryPoint = provideEntryPoint ? stageEntryPoint : undefined;
  let code = getShaderWithEntryPoint(shaderModuleStage, shaderModuleEntryPoint);
  if (extraEntryPoint) {
    code += ` ${getShaderWithEntryPoint(shaderModuleStage, 'extra')}`;
  }
  const descriptor = {
    layout: 'auto',
    vertex: {
      module: t.device.createShaderModule({
        code: kDefaultVertexShaderCode
      })
    },
    fragment: {
      module: t.device.createShaderModule({
        code
      }),
      entryPoint,
      targets: [{ format: 'rgba8unorm' }]
    }
  };
  let _success = true;
  if (shaderModuleStage !== 'fragment') {
    _success = false;
  }
  if (!provideEntryPoint && extraEntryPoint) {
    _success = false;
  }
  if (shaderModuleEntryPoint !== stageEntryPoint && provideEntryPoint) {
    _success = false;
  }
  t.doCreateRenderPipelineTest(isAsync, _success, descriptor);
});

g.test('compute_undefined_entry_point_and_extra_stage').
desc(
  `
Tests calling createComputePipeline(Async) with compute stage shader and
an undefined entryPoint is valid if there's an extra shader stage.
`
).
params((u) =>
u.
combine('isAsync', [true, false]).
combine('extraShaderModuleStage', ['compute', 'vertex', 'fragment'])
).
fn((t) => {
  const { isAsync, extraShaderModuleStage } = t.params;
  const code = `
        ${getShaderWithEntryPoint('compute', 'main')}
        ${getShaderWithEntryPoint(extraShaderModuleStage, 'extra')}
    `;
  const descriptor = {
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({
        code
      }),
      entryPoint: undefined
    }
  };

  const success = extraShaderModuleStage !== 'compute';
  t.doCreateComputePipelineTest(isAsync, success, descriptor);
});

g.test('vertex_undefined_entry_point_and_extra_stage').
desc(
  `
Tests calling createRenderPipeline(Async) with vertex stage shader and
an undefined entryPoint is valid if there's an extra shader stage.
`
).
params((u) =>
u.
combine('isAsync', [true, false]).
combine('extraShaderModuleStage', ['compute', 'vertex', 'fragment'])
).
fn((t) => {
  const { isAsync, extraShaderModuleStage } = t.params;
  const code = `
        ${getShaderWithEntryPoint('vertex', 'main')}
        ${getShaderWithEntryPoint(extraShaderModuleStage, 'extra')}
    `;
  const descriptor = {
    layout: 'auto',
    vertex: {
      module: t.device.createShaderModule({
        code
      }),
      entryPoint: undefined
    }
  };

  const success = extraShaderModuleStage !== 'vertex';
  t.doCreateRenderPipelineTest(isAsync, success, descriptor);
});

g.test('fragment_undefined_entry_point_and_extra_stage').
desc(
  `
Tests calling createRenderPipeline(Async) with fragment stage shader and
an undefined entryPoint is valid if there's an extra shader stage.
`
).
params((u) =>
u.
combine('isAsync', [true, false]).
combine('extraShaderModuleStage', ['compute', 'vertex', 'fragment'])
).
fn((t) => {
  const { isAsync, extraShaderModuleStage } = t.params;
  const code = `
        ${getShaderWithEntryPoint('fragment', 'main')}
        ${getShaderWithEntryPoint(extraShaderModuleStage, 'extra')}
    `;
  const descriptor = {
    layout: 'auto',
    vertex: {
      module: t.device.createShaderModule({
        code: kDefaultVertexShaderCode
      })
    },
    fragment: {
      module: t.device.createShaderModule({
        code
      }),
      entryPoint: undefined,
      targets: [{ format: 'rgba8unorm' }]
    }
  };

  const success = extraShaderModuleStage !== 'fragment';
  t.doCreateRenderPipelineTest(isAsync, success, descriptor);
});