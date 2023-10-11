/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
This test dedicatedly tests validation of GPUMultisampleState of createRenderPipeline.
`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { kDefaultFragmentShaderCode } from '../../../util/shader.js';

import { CreateRenderPipelineValidationTest } from './common.js';

export const g = makeTestGroup(CreateRenderPipelineValidationTest);

g.test('count')
  .desc(`If multisample.count must either be 1 or 4.`)
  .params(u =>
    u
      .combine('isAsync', [false, true])
      .beginSubcases()
      .combine('count', [0, 1, 2, 3, 4, 8, 16, 1024])
  )
  .fn(t => {
    const { isAsync, count } = t.params;

    const descriptor = t.getDescriptor({ multisample: { count, alphaToCoverageEnabled: false } });

    const _success = count === 1 || count === 4;
    t.doCreateRenderPipelineTest(isAsync, _success, descriptor);
  });

g.test('alpha_to_coverage,count')
  .desc(
    `If multisample.alphaToCoverageEnabled is true, multisample.count must be greater than 1, e.g. it can only be 4.`
  )
  .params(u =>
    u
      .combine('isAsync', [false, true])
      .combine('alphaToCoverageEnabled', [false, true])
      .beginSubcases()
      .combine('count', [1, 4])
  )
  .fn(t => {
    const { isAsync, alphaToCoverageEnabled, count } = t.params;

    const descriptor = t.getDescriptor({ multisample: { count, alphaToCoverageEnabled } });

    const _success = alphaToCoverageEnabled ? count === 4 : count === 1 || count === 4;
    t.doCreateRenderPipelineTest(isAsync, _success, descriptor);
  });

g.test('alpha_to_coverage,sample_mask')
  .desc(
    `If sample_mask builtin is a pipeline output of fragment, multisample.alphaToCoverageEnabled should be false.`
  )
  .params(u =>
    u
      .combine('isAsync', [false, true])
      .combine('alphaToCoverageEnabled', [false, true])
      .beginSubcases()
      .combine('hasSampleMaskOutput', [false, true])
  )
  .fn(t => {
    const { isAsync, alphaToCoverageEnabled, hasSampleMaskOutput } = t.params;

    if (t.isCompatibility && hasSampleMaskOutput) {
      t.skip('WGSL sample_mask is not supported in compatibility mode');
    }

    const descriptor = t.getDescriptor({
      multisample: { alphaToCoverageEnabled, count: 4 },
      fragmentShaderCode: hasSampleMaskOutput
        ? `
      struct Output {
        @builtin(sample_mask) mask_out: u32,
        @location(0) color : vec4<f32>,
      }
      @fragment fn main() -> Output {
        var o: Output;
        // We need to make sure this sample_mask isn't optimized out even its value equals "no op".
        o.mask_out = 0xFFFFFFFFu;
        o.color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
        return o;
      }`
        : kDefaultFragmentShaderCode,
    });

    const _success = !hasSampleMaskOutput || !alphaToCoverageEnabled;
    t.doCreateRenderPipelineTest(isAsync, _success, descriptor);
  });
