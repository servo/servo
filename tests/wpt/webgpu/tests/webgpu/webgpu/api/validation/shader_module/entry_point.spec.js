/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
This tests entry point validation of compute/render pipelines and their shader modules.

The entryPoint in shader module include standard "main" and others.
The entryPoint assigned in descriptor include:
- Matching case (control case)
- Empty string
- Mistyping
- Containing invalid char, including space and control codes (Null character)
- Unicode entrypoints and their ASCIIfied version

TODO:
- Test unicode normalization (gpuweb/gpuweb#1160)
- Fine-tune test cases to reduce number by removing trivially similar cases
`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
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
  { shaderModuleEntryPoint: 'séquençage', stageEntryPoint: 'séquençage' },
];

g.test('compute')
  .desc(
    `
Tests calling createComputePipeline(Async) with valid vertex stage shader and different entryPoints,
and check that the APIs only accept matching entryPoint.
`
  )
  .params(u => u.combine('isAsync', [true, false]).combineWithParams(kEntryPointTestCases))
  .fn(t => {
    const { isAsync, shaderModuleEntryPoint, stageEntryPoint } = t.params;
    const descriptor = {
      layout: 'auto',
      compute: {
        module: t.device.createShaderModule({
          code: getShaderWithEntryPoint('compute', shaderModuleEntryPoint),
        }),
        entryPoint: stageEntryPoint,
      },
    };
    const _success = shaderModuleEntryPoint === stageEntryPoint;
    t.doCreateComputePipelineTest(isAsync, _success, descriptor);
  });

g.test('vertex')
  .desc(
    `
Tests calling createRenderPipeline(Async) with valid vertex stage shader and different entryPoints,
and check that the APIs only accept matching entryPoint.
`
  )
  .params(u => u.combine('isAsync', [true, false]).combineWithParams(kEntryPointTestCases))
  .fn(t => {
    const { isAsync, shaderModuleEntryPoint, stageEntryPoint } = t.params;
    const descriptor = {
      layout: 'auto',
      vertex: {
        module: t.device.createShaderModule({
          code: getShaderWithEntryPoint('vertex', shaderModuleEntryPoint),
        }),
        entryPoint: stageEntryPoint,
      },
    };
    const _success = shaderModuleEntryPoint === stageEntryPoint;
    t.doCreateRenderPipelineTest(isAsync, _success, descriptor);
  });

g.test('fragment')
  .desc(
    `
Tests calling createRenderPipeline(Async) with valid fragment stage shader and different entryPoints,
and check that the APIs only accept matching entryPoint.
`
  )
  .params(u => u.combine('isAsync', [true, false]).combineWithParams(kEntryPointTestCases))
  .fn(t => {
    const { isAsync, shaderModuleEntryPoint, stageEntryPoint } = t.params;
    const descriptor = {
      layout: 'auto',
      vertex: {
        module: t.device.createShaderModule({
          code: kDefaultVertexShaderCode,
        }),
        entryPoint: 'main',
      },
      fragment: {
        module: t.device.createShaderModule({
          code: getShaderWithEntryPoint('fragment', shaderModuleEntryPoint),
        }),
        entryPoint: stageEntryPoint,
        targets: [{ format: 'rgba8unorm' }],
      },
    };
    const _success = shaderModuleEntryPoint === stageEntryPoint;
    t.doCreateRenderPipelineTest(isAsync, _success, descriptor);
  });
