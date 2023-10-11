/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Tests limitations of bind group usage in a pipeline in compat mode.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf } from '../../../../../../common/util/data_tables.js';
import { kRenderEncodeTypes } from '../../../../../util/command_buffer_maker.js';
import { CompatibilityTest } from '../../../../compatibility_test.js';

const kTextureTypes = ['regular', 'storage'];

function getTextureTypeWGSL(textureType) {
  return textureType === 'storage' ? 'texture_storage_2d<rgba8unorm, write>' : 'texture_2d<f32>';
}

/**
 * Gets the WGSL needed for testing a render pipeline using texture_2d or texture_storage_2d
 * and either 2 bindgroups or 1
 */
function getRenderShaderModule(device, textureType, bindConfig) {
  const textureTypeWGSL = getTextureTypeWGSL(textureType);
  const secondGroup = bindConfig === 'one bindgroup' ? 0 : 1;
  const secondBinding = secondGroup === 0 ? 1 : 0;
  return device.createShaderModule({
    code: `
      @vertex
      fn vs(@builtin(vertex_index) VertexIndex : u32) -> @builtin(position) vec4f {
        var pos = array(
          vec4f(-1,  3, 0, 1),
          vec4f( 3, -1, 0, 1),
          vec4f(-1, -1, 0, 1));
        return pos[VertexIndex];
      }

      @group(0) @binding(0) var tex0 : ${textureTypeWGSL};
      @group(${secondGroup}) @binding(${secondBinding}) var tex1 : ${textureTypeWGSL};

      @fragment
      fn fs(@builtin(position) pos: vec4f) -> @location(0) vec4f {
        _ = tex0;
        _ = tex1;
        return vec4f(0);
      }
  `,
  });
}

/**
 * Gets the WGSL needed for testing a compute pipeline using texture_2d or texture_storage_2d
 * and either 2 bindgroups or 1
 */
function getComputeShaderModule(device, textureType, bindConfig) {
  const textureTypeWGSL = getTextureTypeWGSL(textureType);
  const secondGroup = bindConfig === 'one bindgroup' ? 0 : 1;
  const secondBinding = secondGroup === 0 ? 1 : 0;
  return device.createShaderModule({
    code: `
      @group(0) @binding(0) var tex0 : ${textureTypeWGSL};
      @group(${secondGroup}) @binding(${secondBinding}) var tex1 : ${textureTypeWGSL};

      @compute @workgroup_size(1)
      fn cs() {
        _ = tex0;
        _ = tex1;
      }
    `,
  });
}

const kBindCases = {
  'incompatible views in the same bindGroup': {
    bindConfig: 'one bindgroup',
    fn(device, pipeline, encoder, texture) {
      const bindGroup = device.createBindGroup({
        layout: pipeline.getBindGroupLayout(0),
        entries: [
          { binding: 0, resource: texture.createView({ baseMipLevel: 0, mipLevelCount: 1 }) },
          { binding: 1, resource: texture.createView({ baseMipLevel: 1, mipLevelCount: 1 }) },
        ],
      });
      encoder.setBindGroup(0, bindGroup);
      return { shouldSucceed: false };
    },
  },
  'incompatible views in different bindGroups': {
    bindConfig: 'two bindgroups',
    fn(device, pipeline, encoder, texture) {
      const bindGroup0 = device.createBindGroup({
        layout: pipeline.getBindGroupLayout(0),
        entries: [
          { binding: 0, resource: texture.createView({ baseMipLevel: 0, mipLevelCount: 1 }) },
        ],
      });
      const bindGroup1 = device.createBindGroup({
        layout: pipeline.getBindGroupLayout(1),
        entries: [
          { binding: 0, resource: texture.createView({ baseMipLevel: 1, mipLevelCount: 1 }) },
        ],
      });
      encoder.setBindGroup(0, bindGroup0);
      encoder.setBindGroup(1, bindGroup1);
      return { shouldSucceed: false };
    },
  },
  'can bind same view in different bindGroups': {
    bindConfig: 'two bindgroups',
    fn(device, pipeline, encoder, texture) {
      const bindGroup0 = device.createBindGroup({
        layout: pipeline.getBindGroupLayout(0),
        entries: [
          { binding: 0, resource: texture.createView({ baseMipLevel: 1, mipLevelCount: 1 }) },
        ],
      });
      const bindGroup1 = device.createBindGroup({
        layout: pipeline.getBindGroupLayout(1),
        entries: [
          { binding: 0, resource: texture.createView({ baseMipLevel: 1, mipLevelCount: 1 }) },
        ],
      });
      encoder.setBindGroup(0, bindGroup0);
      encoder.setBindGroup(1, bindGroup1);
      return { shouldSucceed: true };
    },
  },
  'binding incompatible bindGroups then fix': {
    bindConfig: 'one bindgroup',
    fn(device, pipeline, encoder, texture) {
      const badBindGroup = device.createBindGroup({
        layout: pipeline.getBindGroupLayout(0),
        entries: [
          { binding: 0, resource: texture.createView({ baseMipLevel: 0, mipLevelCount: 1 }) },
          { binding: 1, resource: texture.createView({ baseMipLevel: 1, mipLevelCount: 1 }) },
        ],
      });
      const goodBindGroup = device.createBindGroup({
        layout: pipeline.getBindGroupLayout(0),
        entries: [
          { binding: 0, resource: texture.createView({ baseMipLevel: 1, mipLevelCount: 1 }) },
          { binding: 1, resource: texture.createView({ baseMipLevel: 1, mipLevelCount: 1 }) },
        ],
      });
      encoder.setBindGroup(0, badBindGroup);
      encoder.setBindGroup(0, goodBindGroup);
      return { shouldSucceed: true };
    },
  },
};

function createAndBindTwoBindGroupsWithDifferentViewsOfSameTexture(
  device,
  pipeline,
  encoder,
  texture
) {
  const bindGroup0 = device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [{ binding: 0, resource: texture.createView({ baseMipLevel: 0, mipLevelCount: 1 }) }],
  });
  const bindGroup1 = device.createBindGroup({
    layout: pipeline.getBindGroupLayout(1),
    entries: [{ binding: 0, resource: texture.createView({ baseMipLevel: 1, mipLevelCount: 1 }) }],
  });
  encoder.setBindGroup(0, bindGroup0);
  encoder.setBindGroup(1, bindGroup1);
}

const kBindCaseNames = keysOf(kBindCases);

const kDrawUseCases = {
  draw: (t, encoder) => {
    encoder.draw(3);
  },
  drawIndexed: (t, encoder) => {
    const indexBuffer = t.makeBufferWithContents(new Uint16Array([0, 1, 2]), GPUBufferUsage.INDEX);
    encoder.setIndexBuffer(indexBuffer, 'uint16');
    encoder.drawIndexed(3);
  },
  drawIndirect(t, encoder) {
    const indirectBuffer = t.makeBufferWithContents(
      new Uint32Array([3, 1, 0, 0]),
      GPUBufferUsage.INDIRECT
    );

    encoder.drawIndirect(indirectBuffer, 0);
  },
  drawIndexedIndirect(t, encoder) {
    const indexBuffer = t.makeBufferWithContents(new Uint16Array([0, 1, 2]), GPUBufferUsage.INDEX);
    encoder.setIndexBuffer(indexBuffer, 'uint16');
    const indirectBuffer = t.makeBufferWithContents(
      new Uint32Array([3, 1, 0, 0, 0]),
      GPUBufferUsage.INDIRECT
    );

    encoder.drawIndexedIndirect(indirectBuffer, 0);
  },
};
const kDrawCaseNames = keysOf(kDrawUseCases);

const kDispatchUseCases = {
  dispatchWorkgroups(t, encoder) {
    encoder.dispatchWorkgroups(1);
  },
  dispatchWorkgroupsIndirect(t, encoder) {
    const indirectBuffer = t.makeBufferWithContents(
      new Uint32Array([1, 1, 1]),
      GPUBufferUsage.INDIRECT
    );

    encoder.dispatchWorkgroupsIndirect(indirectBuffer, 0);
  },
};
const kDispatchCaseNames = keysOf(kDispatchUseCases);

function createResourcesForRenderPassTest(t, textureType, bindConfig) {
  const texture = t.device.createTexture({
    size: [2, 1, 1],
    mipLevelCount: 2,
    format: 'rgba8unorm',
    usage:
      textureType === 'storage' ? GPUTextureUsage.STORAGE_BINDING : GPUTextureUsage.TEXTURE_BINDING,
  });
  t.trackForCleanup(texture);

  const module = getRenderShaderModule(t.device, textureType, bindConfig);
  const pipeline = t.device.createRenderPipeline({
    layout: 'auto',
    vertex: {
      module,
      entryPoint: 'vs',
    },
    fragment: {
      module,
      entryPoint: 'fs',
      targets: [{ format: 'rgba8unorm' }],
    },
  });

  return { texture, pipeline };
}

function createResourcesForComputePassTest(t, textureType, bindConfig) {
  const texture = t.device.createTexture({
    size: [2, 1, 1],
    mipLevelCount: 2,
    format: 'rgba8unorm',
    usage:
      textureType === 'storage' ? GPUTextureUsage.STORAGE_BINDING : GPUTextureUsage.TEXTURE_BINDING,
  });
  t.trackForCleanup(texture);

  const module = getComputeShaderModule(t.device, textureType, bindConfig);
  const pipeline = t.device.createComputePipeline({
    layout: 'auto',
    compute: {
      module,
      entryPoint: 'cs',
    },
  });

  return { texture, pipeline };
}

export const g = makeTestGroup(CompatibilityTest);

g.test('twoDifferentTextureViews,render_pass,used')
  .desc(
    `
Tests that you can not use 2 different views of the same texture in a render pass in compat mode.

- Test you can not use incompatible views in the same bindGroup
- Test you can not use incompatible views in different bindGroups
- Test you can bind the same view in different bindGroups
- Test binding incompatible bindGroups is ok as long as they are fixed before draw/dispatch

  The last test is to check validation happens at the correct time (draw/dispatch) and not
  at setBindGroup.
    `
  )
  .params(u =>
    u
      .combine('encoderType', kRenderEncodeTypes)
      .combine('bindCase', kBindCaseNames)
      .combine('useCase', kDrawCaseNames)
      .combine('textureType', kTextureTypes)
      .filter(
        // storage textures can't have 2 bind groups point to the same
        // view even in non-compat. They can have different views in
        // non-compat but not compat.
        p =>
          !(
            p.textureType === 'storage' &&
            (p.bindCase === 'can bind same view in different bindGroups' ||
              p.bindCase === 'binding incompatible bindGroups then fix')
          )
      )
  )
  .fn(t => {
    const { encoderType, bindCase, useCase, textureType } = t.params;
    const { bindConfig, fn } = kBindCases[bindCase];
    const { texture, pipeline } = createResourcesForRenderPassTest(t, textureType, bindConfig);
    const { encoder, validateFinish } = t.createEncoder(encoderType);
    encoder.setPipeline(pipeline);
    const { shouldSucceed } = fn(t.device, pipeline, encoder, texture);
    kDrawUseCases[useCase](t, encoder);
    validateFinish(shouldSucceed);
  });

g.test('twoDifferentTextureViews,render_pass,unused')
  .desc(
    `
Tests that binding 2 different views of the same texture but not using them does not generate a validation error.
    `
  )
  .params(u => u.combine('encoderType', kRenderEncodeTypes).combine('textureType', kTextureTypes))
  .fn(t => {
    const { encoderType, textureType } = t.params;
    const { texture, pipeline } = createResourcesForRenderPassTest(
      t,
      textureType,
      'two bindgroups'
    );

    const { encoder, validateFinish } = t.createEncoder(encoderType);
    encoder.setPipeline(pipeline);
    createAndBindTwoBindGroupsWithDifferentViewsOfSameTexture(t.device, pipeline, encoder, texture);
    validateFinish(true);
  });

g.test('twoDifferentTextureViews,compute_pass,used')
  .desc(
    `
Tests that you can not use 2 different views of the same texture in a compute pass in compat mode.

- Test you can not use incompatible views in the same bindGroup
- Test you can not use incompatible views in different bindGroups
- Test can bind the same view in different bindGroups
- Test that binding incompatible bindGroups is ok as long as they are fixed before draw/dispatch

  The last test is to check validation happens at the correct time (draw/dispatch) and not
  at setBindGroup.
    `
  )
  .params(u =>
    u
      .combine('bindCase', kBindCaseNames)
      .combine('useCase', kDispatchCaseNames)
      .combine('textureType', kTextureTypes)
      .filter(
        // storage textures can't have 2 bind groups point to the same
        // view even in non-compat. They can have different views in
        // non-compat but not compat.
        p =>
          !(
            p.textureType === 'storage' &&
            (p.bindCase === 'can bind same view in different bindGroups' ||
              p.bindCase === 'binding incompatible bindGroups then fix')
          )
      )
  )
  .fn(t => {
    const { bindCase, useCase, textureType } = t.params;
    const { bindConfig, fn } = kBindCases[bindCase];
    const { texture, pipeline } = createResourcesForComputePassTest(t, textureType, bindConfig);
    const { encoder, validateFinish } = t.createEncoder('compute pass');
    encoder.setPipeline(pipeline);
    const { shouldSucceed } = fn(t.device, pipeline, encoder, texture);
    kDispatchUseCases[useCase](t, encoder);
    validateFinish(shouldSucceed);
  });

g.test('twoDifferentTextureViews,compute_pass,unused')
  .desc(
    `
Tests that binding 2 different views of the same texture but not using them does not generate a validation error.
    `
  )
  .params(u => u.combine('textureType', kTextureTypes))
  .fn(t => {
    const { textureType } = t.params;
    const { texture, pipeline } = createResourcesForComputePassTest(
      t,
      textureType,
      'two bindgroups'
    );

    const { encoder, validateFinish } = t.createEncoder('compute pass');
    encoder.setPipeline(pipeline);
    createAndBindTwoBindGroupsWithDifferentViewsOfSameTexture(t.device, pipeline, encoder, texture);
    validateFinish(true);
  });
