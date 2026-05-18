/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Test rendering to 3d texture slices.
- Render to same slice on different render pass, different textures, or texture [1, 1, N]'s different mip levels
- Render to different slices at mip levels on same texture in render pass
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { range } from '../../../../common/util/util.js';
import {

  getColorRenderByteCost,
  isSintOrUintFormat,
  kPossibleColorRenderableTextureFormats,
  textureFormatAndDimensionPossiblyCompatible } from
'../../../format_info.js';
import { AllFeaturesMaxLimitsGPUTest } from '../../../gpu_test.js';
import {
  compareTexelViews,
  getTextureFormatTypeInfo,
  readTextureToTexelViews } from
'../../../shader/execution/expression/call/builtin/texture_utils.js';
import { kTexelRepresentationInfo } from '../../../util/texture/texel_data.js';
import { TexelView } from '../../../util/texture/texel_view.js';

const kSize = 4;

async function checkTextureContent(
t,
{
  texture,
  clearValue,
  mipLevelToSliceToLocationMap




})
{
  const format = texture.format;

  // Generate output values by location for used locations
  const d = isSintOrUintFormat(format) ? 1 : 255;
  const usedLocations = [
  ...new Set([...mipLevelToSliceToLocationMap.values()].flatMap((map) => [...map.values()]))];

  const outputValuesByLocation = new Map(
    usedLocations.map((location) => {
      return [location, range(4, (ch) => outputForLocationByChannel(location, ch) / d)];
    })
  );

  const descriptor = {
    size: [texture.width, texture.height, texture.depthOrArrayLayers],
    dimension: texture.dimension,
    format: texture.format,
    mipLevelCount: texture.mipLevelCount,
    usage: texture.usage
  };

  const actual = await readTextureToTexelViews(t, texture, descriptor, format);
  const zeroTexel = colorNumbersToPerTexelComponent(format, [0, 0, 0, 0]);
  const clearTexel = colorNumbersToPerTexelComponent(format, clearValue);
  const expected = range(descriptor.mipLevelCount, (level) =>
  TexelView.fromTexelsAsColors(
    format,
    (coords) => {
      // If it's not a slice of a mip we rendered to expect 0
      const sliceToLocationMap = mipLevelToSliceToLocationMap.get(level);
      const location = sliceToLocationMap?.get(coords.z);
      if (location === undefined) {
        return zeroTexel;
      }

      // Return the same value the shader would
      const renderTexel = colorNumbersToPerTexelComponent(
        format,
        outputValuesByLocation.get(location)
      );
      return coords.x <= coords.y ? renderTexel : clearTexel;
    }
  )
  );

  const errors = compareTexelViews(t.device, {
    actualTexelViews: actual,
    expectedTexelViews: expected,
    dimension: descriptor.dimension,
    size: descriptor.size
  });
  t.expect(errors.length === 0, `errors in texture: (${texture.label})\n  ${errors.join('\n  ')}`);
}

function getClearValueForFormat(format) {
  return isSintOrUintFormat(format) ? [11, 22, 33, 44] : [0.3, 0.4, 0.5, 0.6];
}

function colorNumbersToPerTexelComponent(
format,
clearValue)
{
  const rep = kTexelRepresentationInfo[format];
  const clearValueAsRep = {};
  const clearRGBA = {
    R: clearValue[0],
    G: clearValue[1],
    B: clearValue[2],
    A: clearValue[3]
  };

  for (const component of rep.componentOrder) {
    clearValueAsRep[component] = clearRGBA[component];
  }
  return clearValueAsRep;
}

// generates:
//    11, 22, 33, 44 for location 0, ch 0, 1, 2, 3
//    21, 32, 43, 54 for location 1, ch 0, 1, 2, 3
//    31, 42, 53, 64 for location 2, ch 0, 1, 2, 3
const outputForLocationByChannel = (location, ch) =>
(ch + location + 1) * 10 + 1 + location;

// Creates a shader module that outputs different values for each location.
// example:
//
//   const d = 255.0;  // d is 1 for integer formats, 255 for floaty formats
//   ...
//   output.color0 = vec4f(11 / d, 21 / d, 31 / d, 41 / d);
//   output.color1 = vec4f(22 / d, 32 / d, 42 / d, 52 / d);
//   output.color2 = vec4f(33 / d, 43 / d, 53 / d, 63 / d);
//   output.color3 = vec4f(44 / d, 54 / d, 64 / d, 74 / d);
//   output.color4 = vec4f(55 / d, 65 / d, 75 / d, 85 / d);
//   output.color5 = vec4f(66 / d, 76 / d, 86 / d, 96 / d);
//   output.color6 = vec4f(77 / d, 87 / d, 97 / d, 107 / d);
//   output.color7 = vec4f(88 / d, 98 / d, 108 / d, 118 / d);
function createShaderModule(t, format, attachmentCount = 1) {
  const { resultType } = getTextureFormatTypeInfo(format);
  const output = (i) =>
  range(4, (ch) => `${outputForLocationByChannel(i, ch)} / d`).join(', ');

  const locations = range(attachmentCount, (i) => `@location(${i}) color${i} : ${resultType}`).join(
    ',\n        '
  );
  const outputs = range(
    attachmentCount,
    (i) => `output.color${i} = ${resultType}(${output(i)});`
  ).join('\n        ');

  const code = `
      struct Output {
        ${locations}
      }

      @vertex
      fn main_vs(@builtin(vertex_index) VertexIndex : u32) -> @builtin(position) vec4f {
        //  -1,1
        //    +-----+
        //    |\\   |
        //    |.\\  |
        //    |..\\ |
        //    |...\\|
        //    +-----+
        // -1,-1   1,-1

        let pos = array(
            // Triangle is slightly extended so its edge doesn't cut through pixel centers.
            vec2f(-1.0, 1.01),
            vec2f(1.01, -1.0),
            vec2f(-1.0, -1.0),
        );
        return vec4f(pos[VertexIndex], 0.0, 1.0);
      }

      const d = ${isSintOrUintFormat(format) ? '1' : '255.0'};

      @fragment
      fn main_fs() -> Output {
        var output : Output;
        ${outputs}
        return output;
      }
  `;
  t.debug(() => code);
  return t.device.createShaderModule({ code });
}

export const g = makeTestGroup(AllFeaturesMaxLimitsGPUTest);

g.test('one_color_attachment,mip_levels').
desc('Render to a 3d texture slice with mip levels.').
params((u) =>
u.
combine('format', kPossibleColorRenderableTextureFormats).
filter((p) => textureFormatAndDimensionPossiblyCompatible('3d', p.format)).
combine('mipLevel', [0, 1, 2]).
combine('depthSlice', [0, 1])
).
fn(async (t) => {
  const { format, mipLevel, depthSlice } = t.params;
  t.skipIfTextureFormatNotSupported(format);
  t.skipIfTextureFormatDoesNotSupportUsage(GPUTextureUsage.RENDER_ATTACHMENT, format);

  const clearValue = getClearValueForFormat(format);

  const descriptor = {
    size: [kSize << mipLevel, kSize << mipLevel, 2 << mipLevel],
    dimension: '3d',
    format,
    mipLevelCount: mipLevel + 1,
    usage:
    GPUTextureUsage.RENDER_ATTACHMENT |
    GPUTextureUsage.COPY_SRC |
    GPUTextureUsage.TEXTURE_BINDING
  };
  const texture = t.createTextureTracked(descriptor);

  const module = createShaderModule(t, format);
  const pipeline = t.device.createRenderPipeline({
    layout: 'auto',
    vertex: { module },
    fragment: {
      module,
      targets: [{ format }]
    },
    primitive: { topology: 'triangle-list' }
  });

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginRenderPass({
    colorAttachments: [
    {
      view: texture.createView({
        baseMipLevel: mipLevel,
        mipLevelCount: 1
      }),
      depthSlice,
      clearValue,
      loadOp: 'clear',
      storeOp: 'store'
    }]

  });
  pass.setPipeline(pipeline);
  pass.draw(3);
  pass.end();
  t.device.queue.submit([encoder.finish()]);

  const sliceToLocationMap = new Map([[depthSlice, 0]]);
  await checkTextureContent(t, {
    texture,
    clearValue,
    mipLevelToSliceToLocationMap: new Map([[mipLevel, sliceToLocationMap]])
  });
});

g.test('multiple_color_attachments,same_mip_level').
desc(
  `
  Render to the different slices of 3d texture in multiple color attachments.
  - Same 3d texture with different slices at same mip level
  - Different 3d textures with same slice at same mip level
  `
).
params((u) =>
u.
combine('format', kPossibleColorRenderableTextureFormats).
filter((p) => textureFormatAndDimensionPossiblyCompatible('3d', p.format)).
combine('sameTexture', [true, false]).
beginSubcases().
combine('samePass', [true, false]).
combine('mipLevel', [0, 1])
).
fn(async (t) => {
  const { format, sameTexture, samePass, mipLevel } = t.params;
  t.skipIfTextureFormatNotSupported(format);
  t.skipIfTextureFormatDoesNotSupportUsage(GPUTextureUsage.RENDER_ATTACHMENT, format);

  const formatByteCost = getColorRenderByteCost(format);
  const maxAttachmentCountPerSample = Math.trunc(
    t.device.limits.maxColorAttachmentBytesPerSample / formatByteCost
  );
  const attachmentCount = Math.min(
    maxAttachmentCountPerSample,
    t.device.limits.maxColorAttachments
  );

  const descriptor = {
    label: 'texture-0',
    size: [kSize << mipLevel, kSize << mipLevel, 1 << attachmentCount << mipLevel],
    dimension: '3d',
    format,
    mipLevelCount: mipLevel + 1,
    usage:
    GPUTextureUsage.RENDER_ATTACHMENT |
    GPUTextureUsage.COPY_SRC |
    GPUTextureUsage.TEXTURE_BINDING
  };

  const clearValue = getClearValueForFormat(format);
  const texture = t.createTextureTracked(descriptor);

  const textures = [texture];
  const sliceToLocationMaps = [new Map()];
  const colorAttachments = [];
  for (let i = 0; i < attachmentCount; i++) {
    let target;
    if (sameTexture) {
      target = texture;
      sliceToLocationMaps[0].set(i, samePass ? i : 0);
    } else {
      descriptor.label = `texture-${i}`;
      const diffTexture = t.createTextureTracked(descriptor);
      textures.push(diffTexture);
      sliceToLocationMaps.push(new Map([[0, samePass ? i : 0]]));
      target = diffTexture;
    }

    const colorAttachment = {
      view: target.createView({
        baseMipLevel: mipLevel,
        mipLevelCount: 1
      }),
      depthSlice: sameTexture ? i : 0,
      clearValue,
      loadOp: 'clear',
      storeOp: 'store'
    };

    colorAttachments.push(colorAttachment);
  }

  const encoder = t.device.createCommandEncoder();

  if (samePass) {
    const module = createShaderModule(t, format, attachmentCount);

    const pipeline = t.device.createRenderPipeline({
      layout: 'auto',
      vertex: { module },
      fragment: {
        module,
        targets: new Array(attachmentCount).fill({ format })
      },
      primitive: { topology: 'triangle-list' }
    });

    const pass = encoder.beginRenderPass({ colorAttachments });
    pass.setPipeline(pipeline);
    pass.draw(3);
    pass.end();
  } else {
    const module = createShaderModule(t, format);

    const pipeline = t.device.createRenderPipeline({
      layout: 'auto',
      vertex: { module },
      fragment: {
        module,
        targets: [{ format }]
      },
      primitive: { topology: 'triangle-list' }
    });

    for (let i = 0; i < attachmentCount; i++) {
      const pass = encoder.beginRenderPass({ colorAttachments: [colorAttachments[i]] });
      pass.setPipeline(pipeline);
      pass.draw(3);
      pass.end();
    }
  }

  t.device.queue.submit([encoder.finish()]);

  await Promise.all(
    textures.map((tex, i) => {
      return checkTextureContent(t, {
        texture: tex,
        clearValue: getClearValueForFormat(format),
        mipLevelToSliceToLocationMap: new Map([[mipLevel, sliceToLocationMaps[i]]])
      });
    })
  );
});

g.test('multiple_color_attachments,same_slice_with_diff_mip_levels').
desc(
  `
  Render to the same slice of a 3d texture at different mip levels in multiple color attachments.
  - For texture size with 1x1xN, the same depth slice of different mip levels can be rendered.
  `
).
params((u) =>
u.
combine('format', kPossibleColorRenderableTextureFormats).
filter((p) => textureFormatAndDimensionPossiblyCompatible('3d', p.format)).
combine('depthSlice', [0, 1])
).
fn(async (t) => {
  const { format, depthSlice } = t.params;
  t.skipIfTextureFormatNotSupported(format);
  t.skipIfTextureFormatDoesNotSupportUsage(GPUTextureUsage.RENDER_ATTACHMENT, format);

  const kBaseSize = 1;

  const formatByteCost = getColorRenderByteCost(format);
  const maxAttachmentCountPerSample = Math.trunc(
    t.device.limits.maxColorAttachmentBytesPerSample / formatByteCost
  );
  const attachmentCount = Math.min(
    maxAttachmentCountPerSample,
    t.device.limits.maxColorAttachments
  );

  const module = createShaderModule(t, format, attachmentCount);

  const pipeline = t.device.createRenderPipeline({
    layout: 'auto',
    vertex: { module },
    fragment: {
      module,
      targets: new Array(attachmentCount).fill({ format })
    },
    primitive: { topology: 'triangle-list' }
  });

  const clearValue = getClearValueForFormat(format);
  const texture = t.createTextureTracked({
    size: [kBaseSize, kBaseSize, depthSlice + 1 << attachmentCount],
    dimension: '3d',
    format,
    mipLevelCount: attachmentCount,
    usage:
    GPUTextureUsage.RENDER_ATTACHMENT |
    GPUTextureUsage.COPY_SRC |
    GPUTextureUsage.TEXTURE_BINDING
  });

  const mipLevelToSliceToLocationMap = new Map();

  const colorAttachments = [];
  for (let i = 0; i < attachmentCount; i++) {
    const sliceToLocationMap = new Map([[depthSlice, i]]);
    mipLevelToSliceToLocationMap.set(i, sliceToLocationMap);
    const colorAttachment = {
      view: texture.createView({
        baseMipLevel: i,
        mipLevelCount: 1
      }),
      depthSlice,
      clearValue,
      loadOp: 'clear',
      storeOp: 'store'
    };

    colorAttachments.push(colorAttachment);
  }

  const encoder = t.device.createCommandEncoder();

  const pass = encoder.beginRenderPass({ colorAttachments });
  pass.setPipeline(pipeline);
  pass.draw(3);
  pass.end();

  t.device.queue.submit([encoder.finish()]);

  await checkTextureContent(t, { texture, clearValue, mipLevelToSliceToLocationMap });
});