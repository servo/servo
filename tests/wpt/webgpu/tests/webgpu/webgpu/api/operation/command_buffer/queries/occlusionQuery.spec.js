/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
API operations tests for occlusion queries.

- test query with
  - scissor
  - sample mask
  - alpha to coverage
  - stencil
  - depth test
- test empty query (no draw) (should be cleared?)
- test via render bundle
- test resolveQuerySet with non-zero firstIndex
- test no queries is zero
- test 0x0 -> 0x3 sample mask
- test 0 -> 1 alpha to coverage
- test resolving twice in same pass keeps values
- test resolving twice across pass keeps values
- test resolveQuerySet destinationOffset
`;import { kUnitCaseParamsBuilder } from '../../../../../common/framework/params_builder.js';
import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import {
  assert,

  range,
  unreachable } from
'../../../../../common/util/util.js';
import { kMaxQueryCount } from '../../../../capability_info.js';

import { GPUTest } from '../../../../gpu_test.js';

const kRequiredQueryBufferOffsetAlignment = 256;
const kBytesPerQuery = 8;
const kTextureSize = [4, 4];

const kRenderModes = ['direct', 'render-bundle'];


const kBufferOffsets = ['zero', 'non-zero'];














// MAINTENANCE_TODO: Refactor these helper classes to use GPUTestBase.createEncoder
//
// The refactor would require some new features in CommandBufferMaker such as:
//
// * Multi render bundle in single render pass support
//
// * Some way to allow calling render pass commands on render bundle encoder.
//   Potentially have a special abstract encoder that wraps the two and defers
//   relevant calls appropriately.

/**
 * This class is used by the RenderPassHelper below to
 * abstract calling these 4 functions on a RenderPassEncoder or a RenderBundleEncoder.
 */











/**
 * This class helps use a render pass encoder or a render bundle encoder
 * in the correct way given the order that operations must happen, in order to be
 * compatible across both paths.
 */
class RenderPassHelper {




  constructor(pass, helper) {
    this._pass = pass;
    this._helper = helper;
  }
  setScissorRect(x, y, width, height) {
    assert(!this._queryHelper);
    this._pass.setScissorRect(x, y, width, height);
  }
  setStencilReference(ref) {
    assert(!this._queryHelper);
    this._pass.setStencilReference(ref);
  }
  beginOcclusionQuery(queryIndex) {
    assert(!this._queryHelper);
    this._pass.beginOcclusionQuery(queryIndex);
    this._queryHelper = this._helper.begin(() => {
      assert(!!this._queryHelper);
      this._queryHelper = undefined;
      this._pass.endOcclusionQuery();
    });
    return this._queryHelper;
  }
}

/**
 * Helper class for using a render pass encoder directly
 */
class QueryHelperDirect {



  constructor(pass, endFn) {
    this._pass = pass;
    this._endFn = endFn;
  }
  setPipeline(pipeline) {
    assert(!!this._pass);
    this._pass.setPipeline(pipeline);
  }
  setVertexBuffer(buffer) {
    assert(!!this._pass);
    this._pass.setVertexBuffer(0, buffer);
  }
  draw(count) {
    assert(!!this._pass);
    this._pass.draw(count);
  }
  end() {
    // make this object impossible to use after calling end
    const fn = this._endFn;
    this._endFn = unreachable;
    this._pass = undefined;
    fn();
  }
}

/**
 * Helper class for starting a query on a render pass encoder directly
 */
class QueryStarterDirect {



  constructor(pass) {
    this._pass = pass;
  }
  begin(endFn) {
    assert(!this._helper);
    this._helper = new QueryHelperDirect(this._pass, () => {
      this._helper = undefined;
      endFn();
    });
    return this._helper;
  }
}

/**
 * Helper class for using a render bundle encoder.
 */
class QueryHelperRenderBundle {



  constructor(pass, endFn) {
    this._encoder = pass;
    this._endFn = endFn;
  }
  setPipeline(pipeline) {
    assert(!!this._encoder);
    this._encoder.setPipeline(pipeline);
  }
  setVertexBuffer(buffer) {
    assert(!!this._encoder);
    this._encoder.setVertexBuffer(0, buffer);
  }
  draw(count) {
    assert(!!this._encoder);
    this._encoder.draw(count);
  }
  end() {
    // make this object impossible to use after calling end
    const fn = this._endFn;
    this._endFn = unreachable;
    this._encoder = undefined;
    fn();
  }
}

/**
 * Helper class for starting a query on a render bundle encoder
 */
class QueryStarterRenderBundle {






  constructor(
  device,
  pass,
  renderPassDescriptor)
  {
    this._device = device;
    this._pass = pass;
    const colorAttachment =
    renderPassDescriptor.colorAttachments[
    0];
    this._renderBundleEncoderDescriptor = {
      colorFormats: ['rgba8unorm'],
      depthStencilFormat: renderPassDescriptor.depthStencilAttachment?.depthLoadOp ?
      'depth24plus' :
      renderPassDescriptor.depthStencilAttachment?.stencilLoadOp ?
      'stencil8' :
      undefined,
      sampleCount: colorAttachment.resolveTarget ? 4 : 1
    };
  }
  begin(endFn) {
    assert(!this._encoder);
    this._encoder = this._device.createRenderBundleEncoder(this._renderBundleEncoderDescriptor);
    this._helper = new QueryHelperRenderBundle(this._encoder, () => {
      assert(!!this._encoder);
      assert(!!this._helper);
      this._pass.executeBundles([this._encoder.finish()]);
      this._helper = undefined;
      this._encoder = undefined;
      endFn();
    });
    return this._helper;
  }
  setPipeline(pipeline) {
    assert(!!this._encoder);
    this._encoder.setPipeline(pipeline);
  }
  setVertexBuffer(buffer) {
    assert(!!this._encoder);
    this._encoder.setVertexBuffer(0, buffer);
  }
  draw(count) {
    assert(!!this._encoder);
    this._encoder.draw(count);
  }
}

class OcclusionQueryTest extends GPUTest {
  createVertexBuffer(data) {
    return this.makeBufferWithContents(data, GPUBufferUsage.VERTEX);
  }
  createSingleTriangleVertexBuffer(z) {

    return this.createVertexBuffer(new Float32Array([
    -0.5, -0.5, z,
    0.5, -0.5, z,
    -0.5, 0.5, z]
    ));
  }
  async readBufferAsBigUint64(buffer) {
    await buffer.mapAsync(GPUMapMode.READ);
    const result = new BigUint64Array(buffer.getMappedRange().slice(0));
    buffer.unmap();
    return result;
  }
  setup(params) {
    const {
      numQueries,
      depthStencilFormat,
      sampleMask = 0xffffffff,
      alpha,
      sampleCount,
      writeMask = 0xf,
      bufferOffset,
      renderMode
    } = params;
    const { device } = this;

    const queryResolveBufferOffset =
    bufferOffset === 'non-zero' ? kRequiredQueryBufferOffsetAlignment : 0;
    const queryResolveBuffer = this.createBufferTracked({
      size: numQueries * 8 + queryResolveBufferOffset,
      usage: GPUBufferUsage.QUERY_RESOLVE | GPUBufferUsage.COPY_SRC
    });

    const readBuffer = this.createBufferTracked({
      size: numQueries * kBytesPerQuery,
      usage: GPUBufferUsage.MAP_READ | GPUBufferUsage.COPY_DST
    });

    const vertexBuffer = this.createSingleTriangleVertexBuffer(0);

    const renderTargetTexture = this.createTextureTracked({
      format: 'rgba8unorm',
      size: kTextureSize,
      usage: GPUTextureUsage.RENDER_ATTACHMENT
    });

    const multisampleRenderTarget = sampleCount ?
    this.createTextureTracked({
      size: kTextureSize,
      format: 'rgba8unorm',
      usage: GPUTextureUsage.RENDER_ATTACHMENT,
      sampleCount
    }) :
    null;

    const depthStencilTexture = depthStencilFormat ?
    this.createTextureTracked({
      format: depthStencilFormat,
      size: kTextureSize,
      usage: GPUTextureUsage.RENDER_ATTACHMENT
    }) :
    undefined;

    const module = device.createShaderModule({
      code: `
        @vertex fn vs(@location(0) pos: vec4f) -> @builtin(position) vec4f {
          return pos;
        }

        @fragment fn fs() -> @location(0) vec4f {
          return vec4f(0, 0, 0, ${alpha === undefined ? 1 : alpha});
        }
      `
    });

    const haveDepth = !!depthStencilFormat && depthStencilFormat.includes('depth');
    const haveStencil = !!depthStencilFormat && depthStencilFormat.includes('stencil');
    assert(!(haveDepth && haveStencil), 'code does not handle mixed depth-stencil');

    const pipeline = device.createRenderPipeline({
      layout: 'auto',
      vertex: {
        module,
        entryPoint: 'vs',
        buffers: [
        {
          arrayStride: 3 * 4,
          attributes: [
          {
            shaderLocation: 0,
            offset: 0,
            format: 'float32x3'
          }]

        }]

      },
      fragment: {
        module,
        entryPoint: 'fs',
        targets: [{ format: 'rgba8unorm', writeMask }]
      },
      ...(sampleCount && {
        multisample: {
          count: sampleCount,
          mask: alpha === undefined ? sampleMask : 0xffffffff,
          alphaToCoverageEnabled: alpha !== undefined
        }
      }),
      ...(depthStencilTexture && {
        depthStencil: {
          format: depthStencilFormat,
          depthWriteEnabled: haveDepth,
          depthCompare: haveDepth ? 'less-equal' : 'always',
          ...(haveStencil && {
            stencilFront: {
              compare: 'equal'
            }
          })
        }
      })
    });

    const querySetOffset = params?.querySetOffset === 'non-zero' ? 7 : 0;
    const occlusionQuerySet = this.createQuerySetTracked({
      type: 'occlusion',
      count: numQueries + querySetOffset
    });

    const renderPassDescriptor = {
      colorAttachments: sampleCount ?
      [
      {
        view: multisampleRenderTarget.createView(),
        resolveTarget: renderTargetTexture.createView(),
        loadOp: 'clear',
        storeOp: 'store'
      }] :

      [
      {
        view: renderTargetTexture.createView(),
        loadOp: 'clear',
        storeOp: 'store'
      }],

      ...(haveDepth && {
        depthStencilAttachment: {
          view: depthStencilTexture.createView(),
          depthLoadOp: 'clear',
          depthStoreOp: 'store',
          depthClearValue: 0.5
        }
      }),
      ...(haveStencil && {
        depthStencilAttachment: {
          view: depthStencilTexture.createView(),
          stencilClearValue: 0,
          stencilLoadOp: 'clear',
          stencilStoreOp: 'store'
        }
      }),
      occlusionQuerySet
    };

    return {
      readBuffer,
      vertexBuffer,
      queryResolveBuffer,
      queryResolveBufferOffset,
      occlusionQuerySet,
      renderTargetTexture,
      renderPassDescriptor,
      pipeline,
      depthStencilTexture,
      querySetOffset,
      renderMode
    };
  }
  async runQueryTest(
  resources,
  renderPassDescriptor,
  encodePassFn,
  checkQueryIndexResultFn)
  {
    const { device } = this;
    const {
      readBuffer,
      queryResolveBuffer,
      queryResolveBufferOffset,
      occlusionQuerySet,
      querySetOffset,
      renderMode = 'direct'
    } = resources;
    const numQueries = occlusionQuerySet.count - querySetOffset;
    const queryIndices = range(numQueries, (i) => i + querySetOffset);

    const encoder = device.createCommandEncoder();
    if (renderPassDescriptor) {
      const pass = encoder.beginRenderPass(renderPassDescriptor);
      const helper = new RenderPassHelper(
        pass,
        renderMode === 'direct' ?
        new QueryStarterDirect(pass) :
        new QueryStarterRenderBundle(device, pass, renderPassDescriptor)
      );

      for (const queryIndex of queryIndices) {
        encodePassFn(helper, queryIndex);
      }
      pass.end();
    }

    encoder.resolveQuerySet(
      occlusionQuerySet,
      querySetOffset,
      numQueries,
      queryResolveBuffer,
      queryResolveBufferOffset
    );
    encoder.copyBufferToBuffer(
      queryResolveBuffer,
      queryResolveBufferOffset,
      readBuffer,
      0,
      readBuffer.size
    );
    device.queue.submit([encoder.finish()]);

    const result = await this.readBufferAsBigUint64(readBuffer);
    for (const queryIndex of queryIndices) {
      const resultNdx = queryIndex - querySetOffset;
      const passed = !!result[resultNdx];
      checkQueryIndexResultFn(passed, queryIndex);
    }

    return result;
  }
}

const kQueryTestBaseParams = kUnitCaseParamsBuilder.
combine('writeMask', [0xf, 0x0]).
combine('renderMode', kRenderModes).
combine('bufferOffset', kBufferOffsets).
combine('querySetOffset', kBufferOffsets);

export const g = makeTestGroup(OcclusionQueryTest);

g.test('occlusion_query,initial').
desc(`Test getting contents of QuerySet without any queries.`).
fn(async (t) => {
  const kNumQueries = kMaxQueryCount;
  const resources = t.setup({ numQueries: kNumQueries });
  await t.runQueryTest(
    resources,
    null,
    () => {},
    (passed) => {
      t.expect(!passed);
    }
  );
});

g.test('occlusion_query,basic').
desc('Test all queries pass').
params(kQueryTestBaseParams).
fn(async (t) => {
  const { writeMask, renderMode, bufferOffset, querySetOffset } = t.params;
  const kNumQueries = 30;
  const resources = t.setup({
    writeMask,
    renderMode,
    bufferOffset,
    querySetOffset,
    numQueries: kNumQueries
  });
  const { renderPassDescriptor, vertexBuffer, pipeline } = resources;

  await t.runQueryTest(
    resources,
    renderPassDescriptor,
    (helper, queryIndex) => {
      const queryHelper = helper.beginOcclusionQuery(queryIndex);
      queryHelper.setPipeline(pipeline);
      queryHelper.setVertexBuffer(vertexBuffer);
      queryHelper.draw(3);
      queryHelper.end();
    },
    (passed, queryIndex) => {
      const expectPassed = true;
      t.expect(
        !!passed === expectPassed,
        `queryIndex: ${queryIndex}, was: ${!!passed}, expected: ${expectPassed}`
      );
    }
  );
});

g.test('occlusion_query,empty').
desc(
  `
      Test beginOcclusionQuery/endOcclusionQuery with nothing in between clears the queries

      Calls beginOcclusionQuery/draw/endOcclusionQuery that should show passing fragments
      and validates they passed. Then executes the same queries (same QuerySet) without drawing.
      Those queries should have not passed.
    `
).
fn(async (t) => {
  const kNumQueries = 30;
  const resources = t.setup({ numQueries: kNumQueries });
  const { vertexBuffer, renderPassDescriptor, pipeline } = resources;

  const makeQueryRunner = (draw) => {
    return (helper, queryIndex) => {
      const queryHelper = helper.beginOcclusionQuery(queryIndex);
      queryHelper.setPipeline(pipeline);
      queryHelper.setVertexBuffer(vertexBuffer);
      if (draw) {
        queryHelper.draw(3);
      }
      queryHelper.end();
    };
  };

  const makeQueryChecker = (draw) => {
    return (passed, queryIndex) => {
      const expectPassed = draw;
      t.expect(
        !!passed === expectPassed,
        `draw: ${draw}, queryIndex: ${queryIndex}, was: ${!!passed}, expected: ${expectPassed}`
      );
    };
  };

  await t.runQueryTest(
    resources,
    renderPassDescriptor,
    makeQueryRunner(true),
    makeQueryChecker(true)
  );
  await t.runQueryTest(
    resources,
    renderPassDescriptor,
    makeQueryRunner(false),
    makeQueryChecker(false)
  );
});

g.test('occlusion_query,scissor').
desc(
  `
      Test beginOcclusionQuery/endOcclusionQuery using scissor to occlude
    `
).
params(kQueryTestBaseParams).
fn(async (t) => {
  const { writeMask, renderMode, bufferOffset, querySetOffset } = t.params;
  const kNumQueries = 30;
  const resources = t.setup({
    writeMask,
    renderMode,
    bufferOffset,
    querySetOffset,
    numQueries: kNumQueries
  });
  const { renderPassDescriptor, renderTargetTexture, vertexBuffer, pipeline } = resources;

  const getScissorRect = (i) => {
    const { width, height } = renderTargetTexture;
    switch (i % 4) {
      case 0: // whole target
        return {
          x: 0,
          y: 0,
          width,
          height,
          occluded: false,
          name: 'whole target'
        };
      case 1: // center
        return {
          x: width / 4,
          y: height / 4,
          width: width / 2,
          height: height / 2,
          occluded: false,
          name: 'center'
        };
      case 2: // none
        return {
          x: width / 4,
          y: height / 4,
          width: 0,
          height: 0,
          occluded: true,
          name: 'none'
        };
      case 3: // top 1/4
        return {
          x: 0,
          y: 0,
          width,
          height: height / 2,
          occluded: true,
          name: 'top quarter'
        };
      default:
        unreachable();
    }
  };

  await t.runQueryTest(
    resources,
    renderPassDescriptor,
    (helper, queryIndex) => {
      const { x, y, width, height } = getScissorRect(queryIndex);
      helper.setScissorRect(x, y, width, height);
      const queryHelper = helper.beginOcclusionQuery(queryIndex);
      queryHelper.setPipeline(pipeline);
      queryHelper.setVertexBuffer(vertexBuffer);
      queryHelper.draw(3);
      queryHelper.end();
    },
    (passed, queryIndex) => {
      const { occluded, name: scissorCase } = getScissorRect(queryIndex);
      const expectPassed = !occluded;
      t.expect(
        !!passed === expectPassed,
        `queryIndex: ${queryIndex}, scissorCase: ${scissorCase}, was: ${!!passed}, expected: ${expectPassed}`
      );
    }
  );
});

g.test('occlusion_query,depth').
desc(
  `
      Test beginOcclusionQuery/endOcclusionQuery using depth test to occlude

      Compares depth against 0.5, with alternating vertex buffers which have a depth
      of 0 and 1. When depth check passes, we expect non-zero successful fragments.
    `
).
params(kQueryTestBaseParams).
fn(async (t) => {
  const { writeMask, renderMode, bufferOffset, querySetOffset } = t.params;
  const kNumQueries = 30;
  const resources = t.setup({
    writeMask,
    renderMode,
    bufferOffset,
    querySetOffset,
    numQueries: kNumQueries,
    depthStencilFormat: 'depth24plus'
  });
  const { vertexBuffer: vertexBufferAtZ0, renderPassDescriptor, pipeline } = resources;
  const vertexBufferAtZ1 = t.createSingleTriangleVertexBuffer(1);

  await t.runQueryTest(
    resources,
    renderPassDescriptor,
    (helper, queryIndex) => {
      const queryHelper = helper.beginOcclusionQuery(queryIndex);
      queryHelper.setPipeline(pipeline);
      queryHelper.setVertexBuffer(queryIndex % 2 ? vertexBufferAtZ1 : vertexBufferAtZ0);
      queryHelper.draw(3);
      queryHelper.end();
    },
    (passed, queryIndex) => {
      const expectPassed = queryIndex % 2 === 0;
      t.expect(
        !!passed === expectPassed,
        `queryIndex: ${queryIndex}, was: ${!!passed}, expected: ${expectPassed}`
      );
    }
  );
});

g.test('occlusion_query,stencil').
desc(
  `
      Test beginOcclusionQuery/endOcclusionQuery using stencil to occlude

      Compares stencil against 0, with alternating stencil reference values of
      of 0 and 1. When stencil test passes, we expect non-zero successful fragments.
    `
).
params(kQueryTestBaseParams).
fn(async (t) => {
  const { writeMask, renderMode, bufferOffset, querySetOffset } = t.params;
  const kNumQueries = 30;
  const resources = t.setup({
    writeMask,
    renderMode,
    bufferOffset,
    querySetOffset,
    numQueries: kNumQueries,
    depthStencilFormat: 'stencil8'
  });
  const { vertexBuffer, renderPassDescriptor, pipeline } = resources;

  await t.runQueryTest(
    resources,
    renderPassDescriptor,
    (helper, queryIndex) => {
      helper.setStencilReference(queryIndex % 2);
      const queryHelper = helper.beginOcclusionQuery(queryIndex);
      queryHelper.setPipeline(pipeline);
      queryHelper.setVertexBuffer(vertexBuffer);
      queryHelper.draw(3);
      queryHelper.end();
    },
    (passed, queryIndex) => {
      const expectPassed = queryIndex % 2 === 0;
      t.expect(
        !!passed === expectPassed,
        `queryIndex: ${queryIndex}, was: ${!!passed}, expected: ${expectPassed}`
      );
    }
  );
});

g.test('occlusion_query,sample_mask').
desc(
  `
      Test beginOcclusionQuery/endOcclusionQuery using sample_mask to occlude

      Set sampleMask to 0, 2, 4, 6 and draw quads in top right or bottom left corners of the texel.
      If the corner we draw to matches the corner masked we expect non-zero successful fragments.

      See: https://learn.microsoft.com/en-us/windows/win32/api/d3d11/ne-d3d11-d3d11_standard_multisample_quality_levels
    `
).
params(kQueryTestBaseParams.combine('sampleMask', [0, 2, 4, 6])).
fn(async (t) => {
  const { writeMask, renderMode, bufferOffset, querySetOffset, sampleMask } = t.params;
  const kNumQueries = 30;
  const sampleCount = 4;
  const resources = t.setup({
    writeMask,
    renderMode,
    bufferOffset,
    querySetOffset,
    numQueries: kNumQueries,
    sampleCount,
    sampleMask
  });
  const { renderPassDescriptor, pipeline } = resources;

  const createQuad = (offset) => {

    return t.createVertexBuffer(new Float32Array([
    offset + 0, offset + 0, 0,
    offset + 0.25, offset + 0, 0,
    offset + 0, offset + 0.25, 0,
    offset + 0, offset + 0.25, 0,
    offset + 0.25, offset + 0, 0,
    offset + 0.25, offset + 0.25, 0]
    ));
  };

  const vertexBufferBL = createQuad(0);
  const vertexBufferTR = createQuad(0.25);

  await t.runQueryTest(
    resources,
    renderPassDescriptor,
    (helper, queryIndex) => {
      const queryHelper = helper.beginOcclusionQuery(queryIndex);
      queryHelper.setPipeline(pipeline);
      queryHelper.setVertexBuffer(queryIndex % 2 ? vertexBufferTR : vertexBufferBL);
      queryHelper.draw(6);
      queryHelper.end();
    },
    (passed, queryIndex) => {
      // Above we draw to a specific corner (sample) of a multi-sampled texel
      // drawMask is the "sampleMask" representation of that corner.
      // In other words, if drawMask is 2 (we drew to the top right) and
      // sampleMask is 2 (drawing is allowed to the top right) then we expect
      // passing fragments.
      const drawMask = queryIndex % 2 ? 2 : 4;
      const expectPassed = !!(sampleMask & drawMask);
      t.expect(
        !!passed === expectPassed,
        `queryIndex: ${queryIndex}, was: ${!!passed}, expected: ${expectPassed}`
      );
    }
  );
});

g.test('occlusion_query,alpha_to_coverage').
desc(
  `
      Test beginOcclusionQuery/endOcclusionQuery using alphaToCoverage to occlude

      Set alpha to 0, 0.25, 0.5, 0.75, and 1, draw quads in 4 corners of texel.
      Some should be culled. We count how many passed via queries. It's undefined which
      will pass but it is defined how many will pass for a given alpha value.

      Note: It seems like the result is well defined but if we find some devices/drivers
      don't follow this exactly then we can relax check for the expected number of passed
      queries.

      See: https://bgolus.medium.com/anti-aliased-alpha-test-the-esoteric-alpha-to-coverage-8b177335ae4f
    `
).
params(kQueryTestBaseParams.combine('alpha', [0, 0.25, 0.5, 0.75, 1.0])).
fn(async (t) => {
  const { writeMask, renderMode, bufferOffset, querySetOffset, alpha } = t.params;
  const kNumQueries = 32;
  const sampleCount = 4;
  const resources = t.setup({
    writeMask,
    renderMode,
    bufferOffset,
    querySetOffset,
    numQueries: kNumQueries,
    sampleCount,
    alpha
  });
  const { renderPassDescriptor, pipeline } = resources;

  const createQuad = (xOffset, yOffset) => {

    return t.createVertexBuffer(new Float32Array([
    xOffset + 0, yOffset + 0, 0,
    xOffset + 0.25, yOffset + 0, 0,
    xOffset + 0, yOffset + 0.25, 0,
    xOffset + 0, yOffset + 0.25, 0,
    xOffset + 0.25, yOffset + 0, 0,
    xOffset + 0.25, yOffset + 0.25, 0]
    ));
  };

  const vertexBuffers = [
  createQuad(0, 0),
  createQuad(0.25, 0),
  createQuad(0, 0.25),
  createQuad(0.25, 0.25)];


  const numPassedPerGroup = new Array(kNumQueries / 4).fill(0);

  // These tests can't use queryIndex to decide what to draw because which mask
  // a particular alpha converts to is implementation defined. When querySetOffset is
  // non-zero the queryIndex will go 7, 8, 9, 10, ... but we need to guarantee
  // 4 queries per pixel and group those results so `queryIndex / 4 | 0` won't work.
  // Instead we count the queries to get 4 draws per group, one to each quadrant of a pixel
  // Then we total up the passes for those 4 queries by queryCount.
  let queryCount = 0;
  let resultCount = 0;
  await t.runQueryTest(
    resources,
    renderPassDescriptor,
    (helper, queryIndex) => {
      const queryHelper = helper.beginOcclusionQuery(queryIndex);
      queryHelper.setPipeline(pipeline);
      queryHelper.setVertexBuffer(vertexBuffers[queryCount++ % 4]);
      queryHelper.draw(6);
      queryHelper.end();
    },
    (passed) => {
      const groupIndex = resultCount++ / 4 | 0;
      numPassedPerGroup[groupIndex] += passed ? 1 : 0;
    }
  );

  const expected = alpha / 0.25 | 0;
  numPassedPerGroup.forEach((numPassed, queryGroup) => {
    t.expect(
      numPassed === expected,
      `queryGroup: ${queryGroup}, was: ${numPassed}, expected: ${expected}`
    );
  });
});

g.test('occlusion_query,multi_resolve').
desc('Test calling resolveQuerySet more than once does not change results').
fn(async (t) => {
  const { device } = t;
  const kNumQueries = 30;
  const {
    pipeline,
    vertexBuffer,
    occlusionQuerySet,
    renderPassDescriptor,
    renderTargetTexture,
    queryResolveBuffer,
    readBuffer
  } = t.setup({ numQueries: kNumQueries });

  const readBuffer2 = t.createBufferTracked(readBuffer);
  const readBuffer3 = t.createBufferTracked(readBuffer);

  const renderSomething = (encoder) => {
    const pass = encoder.beginRenderPass(renderPassDescriptor);
    pass.setPipeline(pipeline);
    pass.setVertexBuffer(0, vertexBuffer);
    pass.setScissorRect(0, 0, renderTargetTexture.width, renderTargetTexture.height);
    pass.draw(3);
    pass.end();
  };

  {
    const encoder = device.createCommandEncoder();
    {
      const pass = encoder.beginRenderPass(renderPassDescriptor);
      pass.setPipeline(pipeline);
      pass.setVertexBuffer(0, vertexBuffer);

      for (let i = 0; i < kNumQueries; ++i) {
        pass.beginOcclusionQuery(i);
        if (i % 2) {
          pass.setScissorRect(0, 0, renderTargetTexture.width, renderTargetTexture.height);
        } else {
          pass.setScissorRect(0, 0, 0, 0);
        }
        pass.draw(3);
        pass.endOcclusionQuery();
      }
      pass.end();
    }

    // Intentionally call resolveQuerySet twice
    encoder.resolveQuerySet(occlusionQuerySet, 0, kNumQueries, queryResolveBuffer, 0);
    encoder.resolveQuerySet(occlusionQuerySet, 0, kNumQueries, queryResolveBuffer, 0);
    encoder.copyBufferToBuffer(queryResolveBuffer, 0, readBuffer, 0, readBuffer.size);

    // Rendering stuff unrelated should not affect results.
    renderSomething(encoder);

    encoder.resolveQuerySet(occlusionQuerySet, 0, kNumQueries, queryResolveBuffer, 0);
    encoder.copyBufferToBuffer(queryResolveBuffer, 0, readBuffer2, 0, readBuffer2.size);
    device.queue.submit([encoder.finish()]);
  }

  // Encode something else and draw again, then read the results
  // They should not be affected.
  {
    const encoder = device.createCommandEncoder();
    renderSomething(encoder);

    encoder.resolveQuerySet(occlusionQuerySet, 0, kNumQueries, queryResolveBuffer, 0);
    encoder.copyBufferToBuffer(queryResolveBuffer, 0, readBuffer3, 0, readBuffer3.size);
    device.queue.submit([encoder.finish()]);
  }

  const results = await Promise.all([
  t.readBufferAsBigUint64(readBuffer),
  t.readBufferAsBigUint64(readBuffer2),
  t.readBufferAsBigUint64(readBuffer3)]
  );

  results.forEach((result, r) => {
    for (let i = 0; i < kNumQueries; ++i) {
      const passed = !!result[i];
      const expectPassed = !!(i % 2);
      t.expect(
        passed === expectPassed,
        `result(${r}): queryIndex: ${i}, passed: ${passed}, expected: ${expectPassed}`
      );
    }
  });
});