/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for the clip_distances extension
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

g.test('use_clip_distances_requires_extension_enabled').
desc(
  `Checks that the clip_distances built-in variable is only allowed with the WGSL extension
     clip_distances enabled in shader and the WebGPU extension clip-distances supported on the
     device.`
).
params((u) =>
u.combine('requireExtension', [true, false]).combine('enableExtension', [true, false])
).
beforeAllSubcases((t) => {
  if (t.params.requireExtension) {
    t.selectDeviceOrSkipTestCase({ requiredFeatures: ['clip-distances'] });
  }
}).
fn((t) => {
  const { requireExtension, enableExtension } = t.params;

  t.expectCompileResult(
    requireExtension && enableExtension,
    `
        ${enableExtension ? 'enable clip_distances;' : ''}
        struct VertexOut {
          @builtin(clip_distances) my_clip_distances : array<f32, 1>,
          @builtin(position) my_position : vec4f,
        }
        @vertex fn main() -> VertexOut {
          var output : VertexOut;
          output.my_clip_distances[0] = 1.0;
          output.my_position = vec4f(0.0, 0.0, 0.0, 1.0);
          return output;
        }
    `
  );
});