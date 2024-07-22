/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { kRenderEncoderTypes, kMaximumLimitBaseParams,
  makeLimitTestGroup } from

'./limit_utils.js';

const kVertexBufferBindGroupPreferences = ['vertexBuffers', 'bindGroups'];


const kLayoutTypes = ['auto', 'explicit'];


/**
 * Given testValue, choose more vertex buffers or more bind groups based on preference
 */
function getNumBindGroupsAndNumVertexBuffers(
device,
preference,
testValue)
{
  switch (preference) {
    case 'bindGroups':{
        const numBindGroups = Math.min(testValue, device.limits.maxBindGroups);
        const numVertexBuffers = Math.max(0, testValue - numBindGroups);
        return { numVertexBuffers, numBindGroups };
      }
    case 'vertexBuffers':{
        const numVertexBuffers = Math.min(testValue, device.limits.maxVertexBuffers);
        const numBindGroups = Math.max(0, testValue - numVertexBuffers);
        return { numVertexBuffers, numBindGroups };
      }
  }
}

function createLayout(device, layoutType, numBindGroups) {
  switch (layoutType) {
    case 'auto':
      return 'auto';
    case 'explicit':{
        const bindGroupLayouts = new Array(numBindGroups);
        if (numBindGroups > 0) {
          bindGroupLayouts.fill(device.createBindGroupLayout({ entries: [] }));
          bindGroupLayouts[numBindGroups - 1] = device.createBindGroupLayout({
            entries: [
            {
              binding: 0,
              visibility: GPUShaderStage.VERTEX,
              buffer: {}
            }]

          });
        }
        return device.createPipelineLayout({ bindGroupLayouts });
      }
  }
}

/**
 * Generate a render pipeline that can be used to test maxBindGroupsPlusVertexBuffers
 */
function getPipelineDescriptor(
device,
preference,
testValue,
layoutType)
{
  // Get the numVertexBuffers and numBindGroups we could use given testValue as a total.
  // We will only use 1 of each but we'll use the last index.
  const { numVertexBuffers, numBindGroups } = getNumBindGroupsAndNumVertexBuffers(
    device,
    preference,
    testValue
  );

  const layout = createLayout(device, layoutType, numBindGroups);

  const [bindGroupDecl, bindGroupUsage] =
  numBindGroups === 0 ?
  ['', ''] :
  [`@group(${numBindGroups - 1}) @binding(0) var<uniform> u: f32;`, `_ = u;`];

  const [attribDecl, attribUsage] =
  numVertexBuffers === 0 ?
  ['', ''] :
  ['@location(0) v: vec4f', `_ = v; // will use vertex buffer ${numVertexBuffers - 1}`];

  const code = `
    ${bindGroupDecl}

    @vertex fn vs(${attribDecl}) -> @builtin(position) vec4f {
      ${bindGroupUsage}
      ${attribUsage}
      return vec4f(0);
    }

    @fragment fn fs() -> @location(0) vec4f {
      return vec4f(0);
    }
  `;

  const module = device.createShaderModule({ code });
  const buffers = new Array(numVertexBuffers);
  if (numVertexBuffers > 0) {
    buffers[numVertexBuffers - 1] = {
      arrayStride: 16,
      attributes: [{ shaderLocation: 0, offset: 0, format: 'float32' }]
    };
  }

  return {
    code,
    descriptor: {
      layout,
      vertex: {
        module,
        buffers
      },
      fragment: {
        module,
        targets: [{ format: 'rgba8unorm' }]
      }
    }
  };
}

const kExtraLimits = {
  maxBindGroups: 'adapterLimit',
  maxVertexBuffers: 'adapterLimit'
};

const limit = 'maxBindGroupsPlusVertexBuffers';
export const { g, description } = makeLimitTestGroup(limit);

g.test('createRenderPipeline,at_over').
desc(`Test using at and over ${limit} limit in createRenderPipeline(Async).`).
params(
  kMaximumLimitBaseParams.
  combine('async', [false, true]).
  beginSubcases().
  combine('preference', kVertexBufferBindGroupPreferences).
  combine('layoutType', kLayoutTypes)
).
fn(async (t) => {
  const { limitTest, testValueName, async, preference, layoutType } = t.params;
  await t.testDeviceWithRequestedMaximumLimits(
    limitTest,
    testValueName,
    async ({ device, testValue, shouldError, actualLimit }) => {
      const maxUsableBindGroupsPlusVertexBuffers =
      device.limits.maxBindGroups + device.limits.maxVertexBuffers;
      t.skipIf(
        maxUsableBindGroupsPlusVertexBuffers < actualLimit,
        `can not test because the max usable bindGroups + vertexBuffers (${maxUsableBindGroupsPlusVertexBuffers}) is < maxBindGroupsAndVertexBuffers (${actualLimit})`
      );
      t.skipIf(
        maxUsableBindGroupsPlusVertexBuffers === actualLimit && testValue > actualLimit,
        `can not test because the max usable bindGroups + vertexBuffers (${maxUsableBindGroupsPlusVertexBuffers}) === maxBindGroupsAndVertexBuffers (${actualLimit})
           but the testValue (${testValue}) > maxBindGroupsAndVertexBuffers (${actualLimit})`
      );

      const { code, descriptor } = getPipelineDescriptor(
        device,
        preference,
        testValue,
        layoutType
      );

      await t.testCreateRenderPipeline(
        descriptor,
        async,
        shouldError,
        `testValue: ${testValue}, actualLimit: ${actualLimit}, shouldError: ${shouldError}\n${code}`
      );
    },
    kExtraLimits
  );
});

g.test('draw,at_over').
desc(`Test using at and over ${limit} limit draw/drawIndexed/drawIndirect/drawIndexedIndirect`).
params(
  kMaximumLimitBaseParams.
  combine('encoderType', kRenderEncoderTypes).
  beginSubcases().
  combine('preference', kVertexBufferBindGroupPreferences).
  combine('drawType', ['draw', 'drawIndexed', 'drawIndirect', 'drawIndexedIndirect'])
).
fn(async (t) => {
  const { limitTest, testValueName, encoderType, drawType, preference } = t.params;
  await t.testDeviceWithRequestedMaximumLimits(
    limitTest,
    testValueName,
    async ({ device, testValue, shouldError, actualLimit }) => {
      const maxUsableVertexBuffers = Math.min(
        device.limits.maxVertexBuffers,
        device.limits.maxVertexAttributes
      );
      const maxUsableBindGroupsPlusVertexBuffers =
      device.limits.maxBindGroups + maxUsableVertexBuffers;
      t.skipIf(
        maxUsableBindGroupsPlusVertexBuffers < actualLimit,
        `can not test because the max usable bindGroups + vertexBuffers (${maxUsableBindGroupsPlusVertexBuffers}) is < maxBindGroupsAndVertexBuffers (${actualLimit})`
      );
      t.skipIf(
        maxUsableBindGroupsPlusVertexBuffers === actualLimit && testValue > actualLimit,
        `can not test because the max usable bindGroups + vertexBuffers (${maxUsableBindGroupsPlusVertexBuffers}) === maxBindGroupsAndVertexBuffers (${actualLimit})
           but the testValue (${testValue}) > maxBindGroupsAndVertexBuffers (${actualLimit})`
      );

      // Get the numVertexBuffers and numBindGroups we could use given testValue as a total.
      // We will only use 1 of each but we'll use the last index.
      const { numVertexBuffers, numBindGroups } = getNumBindGroupsAndNumVertexBuffers(
        device,
        preference,
        testValue
      );

      const module = device.createShaderModule({
        code: `
            @vertex fn vs() -> @builtin(position) vec4f {
              return vec4f(0);
            }
            @fragment fn fs() -> @location(0) vec4f {
              return vec4f(0);
            }
            `
      });
      const pipeline = device.createRenderPipeline({
        layout: 'auto',
        vertex: { module },
        fragment: { module, targets: [{ format: 'rgba8unorm' }] }
      });

      const vertexBuffer = t.createBufferTracked({
        size: 16,
        usage: GPUBufferUsage.VERTEX
      });

      await t.testGPURenderAndBindingCommandsMixin(
        encoderType,
        ({ bindGroup, passEncoder }) => {
          // Set the last vertex buffer and clear it. This should have no effect
          // unless there is a bug in bookkeeping in the implementation.
          passEncoder.setVertexBuffer(device.limits.maxVertexBuffers - 1, vertexBuffer);
          passEncoder.setVertexBuffer(device.limits.maxVertexBuffers - 1, null);

          // Set the last bindGroup and clear it. This should have no effect
          // unless there is a bug in bookkeeping in the implementation.
          passEncoder.setBindGroup(device.limits.maxBindGroups - 1, bindGroup);
          passEncoder.setBindGroup(device.limits.maxBindGroups - 1, null);

          if (numVertexBuffers) {
            passEncoder.setVertexBuffer(numVertexBuffers - 1, vertexBuffer);
          }

          if (numBindGroups) {
            passEncoder.setBindGroup(numBindGroups - 1, bindGroup);
          }

          passEncoder.setPipeline(pipeline);

          const indirectBuffer = t.createBufferTracked({
            size: 20,
            usage: GPUBufferUsage.INDIRECT
          });

          const indexBuffer = t.createBufferTracked({
            size: 4,
            usage: GPUBufferUsage.INDEX
          });

          switch (drawType) {
            case 'draw':
              passEncoder.draw(0);
              break;
            case 'drawIndexed':
              passEncoder.setIndexBuffer(indexBuffer, 'uint16');
              passEncoder.drawIndexed(0);
              break;
            case 'drawIndirect':
              passEncoder.drawIndirect(indirectBuffer, 0);
              break;
            case 'drawIndexedIndirect':
              passEncoder.setIndexBuffer(indexBuffer, 'uint16');
              passEncoder.drawIndexedIndirect(indirectBuffer, 0);
              break;
          }
        },
        shouldError,
        `testValue: ${testValue}, actualLimit: ${actualLimit}, shouldError: ${shouldError}`
      );

      vertexBuffer.destroy();
    },
    kExtraLimits
  );
});