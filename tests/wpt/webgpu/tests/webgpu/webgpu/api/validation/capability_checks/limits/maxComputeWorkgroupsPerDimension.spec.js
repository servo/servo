/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { kMaximumLimitBaseParams, makeLimitTestGroup } from './limit_utils.js';const limit = 'maxComputeWorkgroupsPerDimension';
export const { g, description } = makeLimitTestGroup(limit);

const kCreateComputePipelineTypes = [
'createComputePipeline',
'createComputePipelineAsync'];



async function createComputePipeline(
device,
descriptor,
pipelineType)
{
  switch (pipelineType) {
    case 'createComputePipeline':
      return device.createComputePipeline(descriptor);
    case 'createComputePipelineAsync':
      return await device.createComputePipelineAsync(descriptor);
  }
}

// Note: dispatchWorkgroupsIndirect is not tested because it's not a validation error if that exceeds the limits
g.test('dispatchWorkgroups,at_over').
desc(`Test using dispatchWorkgroups at and over ${limit} limit`).
params(
  kMaximumLimitBaseParams.
  combine('pipelineType', kCreateComputePipelineTypes).
  combine('axis', [0, 1, 2])
).
fn(async (t) => {
  const { limitTest, testValueName, pipelineType, axis } = t.params;
  await t.testDeviceWithRequestedMaximumLimits(
    limitTest,
    testValueName,
    async ({ device, testValue, shouldError }) => {
      const counts = [1, 1, 1];
      counts[axis] = testValue;

      const buffer = t.createBufferTracked({
        size: 16,
        usage: GPUBufferUsage.STORAGE
      });

      const module = device.createShaderModule({
        code: `
          @compute @workgroup_size(1) fn main() {
          }
          `
      });

      const pipeline = await createComputePipeline(
        device,
        {
          layout: 'auto',
          compute: {
            module,
            entryPoint: 'main'
          }
        },
        pipelineType
      );

      const encoder = device.createCommandEncoder();
      const pass = encoder.beginComputePass();
      pass.setPipeline(pipeline);
      pass.dispatchWorkgroups(counts[0], counts[1], counts[2]);
      pass.end();

      await t.expectValidationError(() => {
        encoder.finish();
      }, shouldError);

      buffer.destroy();
    }
  );
});

g.test('validate').
desc(
  `Test that ${limit} <= maxComputeWorkgroupSizeX x maxComputeWorkgroupSizeY x maxComputeWorkgroupSizeZ`
).
fn((t) => {
  const { adapter, defaultLimit, adapterLimit } = t;
  const defaultMaxComputeWorkgroupSizeProduct =
  t.getDefaultLimit('maxComputeWorkgroupSizeX') *
  t.getDefaultLimit('maxComputeWorkgroupSizeY') *
  t.getDefaultLimit('maxComputeWorkgroupSizeZ');
  const maxComputeWorkgroupSizeProduct =
  adapter.limits.maxComputeWorkgroupSizeX *
  adapter.limits.maxComputeWorkgroupSizeY *
  adapter.limits.maxComputeWorkgroupSizeZ;
  t.expect(defaultLimit <= defaultMaxComputeWorkgroupSizeProduct);
  t.expect(adapterLimit <= maxComputeWorkgroupSizeProduct);
});