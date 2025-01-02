/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests limitations of createRenderPipeline related to vertex state in compat mode.
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { range } from '../../../../../common/util/util.js';
import { CompatibilityTest } from '../../../compatibility_test.js';

export const g = makeTestGroup(CompatibilityTest);

g.test('maxVertexAttributesVertexIndexInstanceIndex').
desc(
  `
Tests @builtin(vertex_index) and @builtin(instance_index) each count as an attribute.

- Test that you can use maxVertexAttributes
- Test that you can not use maxVertexAttributes and @builtin(vertex_index)
- Test that you can not use maxVertexAttributes and @builtin(instance_index)
- Test that you can use maxVertexAttributes - 1 and @builtin(vertex_index)
- Test that you can use maxVertexAttributes - 1 and @builtin(instance_index)
- Test that you can not use maxVertexAttributes - 1 and both @builtin(vertex_index) and @builtin(instance_index)
- Test that you can use maxVertexAttributes - 2 and both @builtin(vertex_index) and @builtin(instance_index)
    `
).
params((u) =>
u.
combine('useVertexIndex', [false, true]).
combine('useInstanceIndex', [false, true]).
combine('numAttribsToReserve', [0, 1, 2]).
combine('isAsync', [false, true])
).
fn((t) => {
  const { useVertexIndex, useInstanceIndex, numAttribsToReserve, isAsync } = t.params;
  const numAttribs = t.device.limits.maxVertexAttributes - numAttribsToReserve;

  const numBuiltinsUsed = (useVertexIndex ? 1 : 0) + (useInstanceIndex ? 1 : 0);
  const isValidInCompat = numAttribs + numBuiltinsUsed <= t.device.limits.maxVertexAttributes;
  const isValidInCore = numAttribs <= t.device.limits.maxVertexAttributes;
  const isValid = t.isCompatibility ? isValidInCompat : isValidInCore;

  const inputs = range(numAttribs, (i) => `@location(${i}) v${i}: vec4f`);
  const outputs = range(numAttribs, (i) => `v${i}`);

  if (useVertexIndex) {
    inputs.push('@builtin(vertex_index) vNdx: u32');
    outputs.push('vec4f(f32(vNdx))');
  }

  if (useInstanceIndex) {
    inputs.push('@builtin(instance_index) iNdx: u32');
    outputs.push('vec4f(f32(iNdx))');
  }

  const module = t.device.createShaderModule({
    code: `
        @fragment fn fs() -> @location(0) vec4f {
            return vec4f(1);
        }
        @vertex fn vs(${inputs.join(', ')}) -> @builtin(position) vec4f {
            return ${outputs.join(' + ')};
        }
      `
  });

  const pipelineDescriptor = {
    layout: 'auto',
    vertex: {
      module,
      entryPoint: 'vs',
      buffers: [
      {
        arrayStride: 16,
        attributes: range(numAttribs, (i) => ({
          shaderLocation: i,
          format: 'float32x4',
          offset: 0
        }))
      }]

    },
    fragment: {
      module,
      entryPoint: 'fs',
      targets: [
      {
        format: 'rgba8unorm'
      }]

    }
  };

  t.doCreateRenderPipelineTest(isAsync, isValid, pipelineDescriptor);
});