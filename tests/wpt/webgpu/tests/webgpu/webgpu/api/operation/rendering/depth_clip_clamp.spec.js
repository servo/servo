/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Tests for depth clipping, depth clamping (at various points in the pipeline), and maybe extended
depth ranges as well.
`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { kDepthStencilFormats, kTextureFormatInfo } from '../../../format_info.js';
import { GPUTest } from '../../../gpu_test.js';
import { checkElementsBetween, checkElementsPassPredicate } from '../../../util/check_contents.js';

export const g = makeTestGroup(GPUTest);

g.test('depth_clamp_and_clip')
  .desc(
    `
Depth written to the depth attachment should always be in the range of the viewport depth,
even if it was written by the fragment shader (using frag_depth). If depth clipping is enabled,
primitives should be clipped to the viewport depth before rasterization; if not, these fragments
should be rasterized, and the fragment shader should receive out-of-viewport position.z values.

To test this, render NxN points, with N vertex depth values, by (if writeDepth=true) N
frag_depth values with the viewport depth set to [0.25,0.75].

While rendering, check the fragment input position.z has the expected value (for all fragments that
were produced by the rasterizer) by writing the diff to a storage buffer, which is later checked to
be all (near) 0.

Then, run another pass (which outputs every point at z=0.5 to avoid clipping) to verify the depth
buffer contents by outputting the expected depth with depthCompare:'not-equal': any fragments that
have unexpected values then get drawn to the color buffer, which is later checked to be empty.`
  )
  .params(u =>
    u //
      .combine('format', kDepthStencilFormats)
      .filter(p => !!kTextureFormatInfo[p.format].depth)
      .combine('unclippedDepth', [undefined, false, true])
      .combine('writeDepth', [false, true])
      .combine('multisampled', [false, true])
  )
  .beforeAllSubcases(t => {
    const info = kTextureFormatInfo[t.params.format];

    t.selectDeviceOrSkipTestCase([
      t.params.unclippedDepth ? 'depth-clip-control' : undefined,
      info.feature,
    ]);
  })
  .fn(async t => {
    const { format, unclippedDepth, writeDepth, multisampled } = t.params;
    const info = kTextureFormatInfo[format];

    /** Number of depth values to test for both vertex output and frag_depth output. */
    const kNumDepthValues = 8;
    /** Test every combination of vertex output and frag_depth output. */
    const kNumTestPoints = kNumDepthValues * kNumDepthValues;
    const kViewportMinDepth = 0.25;
    const kViewportMaxDepth = 0.75;

    const shaderSource = `
      // Test depths, with viewport range corresponding to [0,1].
      var<private> kDepths: array<f32, ${kNumDepthValues}> = array<f32, ${kNumDepthValues}>(
          -1.0, -0.5, 0.0, 0.25, 0.75, 1.0, 1.5, 2.0);

      const vpMin: f32 = ${kViewportMinDepth};
      const vpMax: f32 = ${kViewportMaxDepth};

      // Draw the points in a straight horizontal row, one per pixel.
      fn vertexX(idx: u32) -> f32 {
        return (f32(idx) + 0.5) * 2.0 / ${kNumTestPoints}.0 - 1.0;
      }

      // Test vertex shader's position.z output.
      // Here, the viewport range corresponds to position.z in [0,1].
      fn vertexZ(idx: u32) -> f32 {
        return kDepths[idx / ${kNumDepthValues}u];
      }

      // Test fragment shader's expected position.z input.
      // Here, the viewport range corresponds to position.z in [vpMin,vpMax], but
      // unclipped values extend beyond that range.
      fn expectedFragPosZ(idx: u32) -> f32 {
        return vpMin + vertexZ(idx) * (vpMax - vpMin);
      }

      //////// "Test" entry points

      struct VFTest {
        @builtin(position) pos: vec4<f32>,
        @location(0) @interpolate(flat) vertexIndex: u32,
      };

      @vertex
      fn vtest(@builtin(vertex_index) idx: u32) -> VFTest {
        var vf: VFTest;
        vf.pos = vec4<f32>(vertexX(idx), 0.0, vertexZ(idx), 1.0);
        vf.vertexIndex = idx;
        return vf;
      }

      struct Output {
        // Each fragment (that didn't get clipped) writes into one element of this output.
        // (Anything that doesn't get written is already zero.)
        fragInputZDiff: array<f32, ${kNumTestPoints}>
      };
      @group(0) @binding(0) var <storage, read_write> output: Output;

      fn checkZ(vf: VFTest) {
        output.fragInputZDiff[vf.vertexIndex] = vf.pos.z - expectedFragPosZ(vf.vertexIndex);
      }

      @fragment
      fn ftest_WriteDepth(vf: VFTest) -> @builtin(frag_depth) f32 {
        checkZ(vf);
        return kDepths[vf.vertexIndex % ${kNumDepthValues}u];
      }

      @fragment
      fn ftest_NoWriteDepth(vf: VFTest) {
        checkZ(vf);
      }

      //////// "Check" entry points

      struct VFCheck {
        @builtin(position) pos: vec4<f32>,
        @location(0) @interpolate(flat) vertexIndex: u32,
      };

      @vertex
      fn vcheck(@builtin(vertex_index) idx: u32) -> VFCheck {
        var vf: VFCheck;
        // Depth=0.5 because we want to render every point, not get clipped.
        vf.pos = vec4<f32>(vertexX(idx), 0.0, 0.5, 1.0);
        vf.vertexIndex = idx;
        return vf;
      }

      struct FCheck {
        @builtin(frag_depth) depth: f32,
        @location(0) color: f32,
      };

      @fragment
      fn fcheck(vf: VFCheck) -> FCheck {
        let vertZ = vertexZ(vf.vertexIndex);
        let outOfRange = vertZ < 0.0 || vertZ > 1.0;
        let expFragPosZ = expectedFragPosZ(vf.vertexIndex);

        let writtenDepth = kDepths[vf.vertexIndex % ${kNumDepthValues}u];

        let expectedDepthWriteInput = ${writeDepth ? 'writtenDepth' : 'expFragPosZ'};
        var expectedDepthBufferValue = clamp(expectedDepthWriteInput, vpMin, vpMax);
        if (${!unclippedDepth} && outOfRange) {
          // Test fragment should have been clipped; expect the depth attachment to
          // have its clear value (0.5).
          expectedDepthBufferValue = 0.5;
        }

        var f: FCheck;
        f.depth = expectedDepthBufferValue;
        f.color = 1.0; // Color written if the resulting depth is unexpected.
        return f;
      }
    `;
    const module = t.device.createShaderModule({ code: shaderSource });

    // Draw points at different vertex depths and fragment depths into the depth attachment,
    // with a viewport of [0.25,0.75].
    const testPipeline = t.device.createRenderPipeline({
      layout: 'auto',
      vertex: { module, entryPoint: 'vtest' },
      primitive: {
        topology: 'point-list',
        unclippedDepth,
      },
      depthStencil: { format, depthWriteEnabled: true, depthCompare: 'always' },
      multisample: multisampled ? { count: 4 } : undefined,
      fragment: {
        module,
        entryPoint: writeDepth ? 'ftest_WriteDepth' : 'ftest_NoWriteDepth',
        targets: [],
      },
    });

    // Use depth comparison to check that the depth attachment now has the expected values.
    const checkPipeline = t.device.createRenderPipeline({
      layout: 'auto',
      vertex: { module, entryPoint: 'vcheck' },
      primitive: { topology: 'point-list' },
      depthStencil: {
        format,
        // NOTE: This check is probably very susceptible to floating point error. If it fails, maybe
        // replace it with two checks (less + greater) with an epsilon applied in the check shader?
        depthCompare: 'not-equal', // Expect every depth value to be exactly equal.
        depthWriteEnabled: true, // If the check failed, overwrite with the expected result.
      },
      multisample: multisampled ? { count: 4 } : undefined,
      fragment: { module, entryPoint: 'fcheck', targets: [{ format: 'r8unorm' }] },
    });

    const dsTexture = t.device.createTexture({
      format,
      size: [kNumTestPoints],
      usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC,
      sampleCount: multisampled ? 4 : 1,
    });
    const dsTextureView = dsTexture.createView();

    const checkTextureDesc = {
      format: 'r8unorm',
      size: [kNumTestPoints],
      usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC,
    };
    const checkTexture = t.device.createTexture(checkTextureDesc);
    const checkTextureView = checkTexture.createView();
    const checkTextureMSView = multisampled
      ? t.device.createTexture({ ...checkTextureDesc, sampleCount: 4 }).createView()
      : undefined;

    const dsActual =
      !multisampled && info.bytesPerBlock
        ? t.device.createBuffer({
            size: kNumTestPoints * info.bytesPerBlock,
            usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
          })
        : undefined;
    const dsExpected =
      !multisampled && info.bytesPerBlock
        ? t.device.createBuffer({
            size: kNumTestPoints * info.bytesPerBlock,
            usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
          })
        : undefined;
    const checkBuffer = t.device.createBuffer({
      size: kNumTestPoints,
      usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
    });

    const fragInputZFailedBuffer = t.device.createBuffer({
      size: 4 * kNumTestPoints,
      usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC,
    });
    const testBindGroup = t.device.createBindGroup({
      layout: testPipeline.getBindGroupLayout(0),
      entries: [{ binding: 0, resource: { buffer: fragInputZFailedBuffer } }],
    });

    const enc = t.device.createCommandEncoder();
    {
      const pass = enc.beginRenderPass({
        colorAttachments: [],
        depthStencilAttachment: {
          view: dsTextureView,
          depthClearValue: 0.5, // Will see this depth value if the fragment was clipped.
          depthLoadOp: 'clear',
          depthStoreOp: 'store',
          stencilClearValue: info.stencil ? 0 : undefined,
          stencilLoadOp: info.stencil ? 'clear' : undefined,
          stencilStoreOp: info.stencil ? 'discard' : undefined,
        },
      });
      pass.setPipeline(testPipeline);
      pass.setBindGroup(0, testBindGroup);
      pass.setViewport(0, 0, kNumTestPoints, 1, kViewportMinDepth, kViewportMaxDepth);
      pass.draw(kNumTestPoints);
      pass.end();
    }
    if (dsActual) {
      enc.copyTextureToBuffer({ texture: dsTexture }, { buffer: dsActual }, [kNumTestPoints]);
    }
    {
      const clearValue = [0, 0, 0, 0]; // Will see this color if the check passed.
      const pass = enc.beginRenderPass({
        colorAttachments: [
          checkTextureMSView
            ? {
                view: checkTextureMSView,
                resolveTarget: checkTextureView,
                clearValue,
                loadOp: 'clear',
                storeOp: 'discard',
              }
            : { view: checkTextureView, clearValue, loadOp: 'clear', storeOp: 'store' },
        ],

        depthStencilAttachment: {
          view: dsTextureView,
          depthLoadOp: 'load',
          depthStoreOp: 'store',
          stencilClearValue: info.stencil ? 0 : undefined,
          stencilLoadOp: info.stencil ? 'clear' : undefined,
          stencilStoreOp: info.stencil ? 'discard' : undefined,
        },
      });
      pass.setPipeline(checkPipeline);
      pass.setViewport(0, 0, kNumTestPoints, 1, 0.0, 1.0);
      pass.draw(kNumTestPoints);
      pass.end();
    }
    enc.copyTextureToBuffer({ texture: checkTexture }, { buffer: checkBuffer }, [kNumTestPoints]);
    if (dsExpected) {
      enc.copyTextureToBuffer({ texture: dsTexture }, { buffer: dsExpected }, [kNumTestPoints]);
    }
    t.device.queue.submit([enc.finish()]);

    t.expectGPUBufferValuesPassCheck(
      fragInputZFailedBuffer,
      a => checkElementsBetween(a, [() => -1e-5, () => 1e-5]),
      { type: Float32Array, typedLength: kNumTestPoints }
    );

    const kCheckPassedValue = 0;
    const predicatePrinter = [
      { leftHeader: 'expected ==', getValueForCell: index => kCheckPassedValue },
    ];

    if (dsActual && dsExpected && format === 'depth32float') {
      await Promise.all([dsActual.mapAsync(GPUMapMode.READ), dsExpected.mapAsync(GPUMapMode.READ)]);
      const act = new Float32Array(dsActual.getMappedRange());
      const exp = new Float32Array(dsExpected.getMappedRange());
      predicatePrinter.push(
        { leftHeader: 'act ==', getValueForCell: index => act[index].toFixed(2) },
        { leftHeader: 'exp ==', getValueForCell: index => exp[index].toFixed(2) }
      );
    }
    t.expectGPUBufferValuesPassCheck(
      checkBuffer,
      a =>
        checkElementsPassPredicate(a, (index, value) => value === kCheckPassedValue, {
          predicatePrinter,
        }),
      { type: Uint8Array, typedLength: kNumTestPoints, method: 'map' }
    );
  });

g.test('depth_test_input_clamped')
  .desc(
    `
Input to the depth test should always be in the range of viewport depth, even if it was written by
the fragment shader (using frag_depth).

To test this, first initialize the depth buffer with N expected values (by writing frag_depth, with
the default viewport). These expected values are clamped by the shader to [0.25, 0.75].

Then, run another pass with the viewport depth set to [0.25,0.75], and output various (unclamped)
frag_depth values from its fragment shader with depthCompare:'not-equal'. These should get clamped;
any fragments that have unexpected values then get drawn to the color buffer, which is later checked
to be empty.`
  )
  .params(u =>
    u //
      .combine('format', kDepthStencilFormats)
      .filter(p => !!kTextureFormatInfo[p.format].depth)
      .combine('unclippedDepth', [false, true])
      .combine('multisampled', [false, true])
  )
  .beforeAllSubcases(t => {
    const info = kTextureFormatInfo[t.params.format];

    t.selectDeviceOrSkipTestCase([
      t.params.unclippedDepth ? 'depth-clip-control' : undefined,
      info.feature,
    ]);
  })
  .fn(t => {
    const { format, unclippedDepth, multisampled } = t.params;
    const info = kTextureFormatInfo[format];

    const kNumDepthValues = 8;
    const kViewportMinDepth = 0.25;
    const kViewportMaxDepth = 0.75;

    const shaderSource = `
      // Test depths, with viewport range corresponding to [0,1].
      var<private> kDepths: array<f32, ${kNumDepthValues}> = array<f32, ${kNumDepthValues}>(
          -1.0, -0.5, 0.0, 0.25, 0.75, 1.0, 1.5, 2.0);

      const vpMin: f32 = ${kViewportMinDepth};
      const vpMax: f32 = ${kViewportMaxDepth};

      // Draw the points in a straight horizontal row, one per pixel.
      fn vertexX(idx: u32) -> f32 {
        return (f32(idx) + 0.5) * 2.0 / ${kNumDepthValues}.0 - 1.0;
      }

      struct VF {
        @builtin(position) pos: vec4<f32>,
        @location(0) @interpolate(flat) vertexIndex: u32,
      };

      @vertex
      fn vmain(@builtin(vertex_index) idx: u32) -> VF {
        var vf: VF;
        // Depth=0.5 because we want to render every point, not get clipped.
        vf.pos = vec4<f32>(vertexX(idx), 0.0, 0.5, 1.0);
        vf.vertexIndex = idx;
        return vf;
      }

      @fragment
      fn finit(vf: VF) -> @builtin(frag_depth) f32 {
        // Expected values of the ftest pipeline.
        return clamp(kDepths[vf.vertexIndex], vpMin, vpMax);
      }

      struct FTest {
        @builtin(frag_depth) depth: f32,
        @location(0) color: f32,
      };

      @fragment
      fn ftest(vf: VF) -> FTest {
        var f: FTest;
        f.depth = kDepths[vf.vertexIndex]; // Should get clamped to the viewport.
        f.color = 1.0; // Color written if the resulting depth is unexpected.
        return f;
      }
    `;

    const module = t.device.createShaderModule({ code: shaderSource });

    // Initialize depth attachment with expected values, in [0.25,0.75].
    const initPipeline = t.device.createRenderPipeline({
      layout: 'auto',
      vertex: { module, entryPoint: 'vmain' },
      primitive: { topology: 'point-list' },
      depthStencil: { format, depthWriteEnabled: true, depthCompare: 'always' },
      multisample: multisampled ? { count: 4 } : undefined,
      fragment: { module, entryPoint: 'finit', targets: [] },
    });

    // With a viewport set to [0.25,0.75], output values in [0.0,1.0] and check they're clamped
    // before the depth test, regardless of whether unclippedDepth is enabled.
    const testPipeline = t.device.createRenderPipeline({
      layout: 'auto',
      vertex: { module, entryPoint: 'vmain' },
      primitive: {
        topology: 'point-list',
        unclippedDepth,
      },
      depthStencil: { format, depthCompare: 'not-equal', depthWriteEnabled: false },
      multisample: multisampled ? { count: 4 } : undefined,
      fragment: { module, entryPoint: 'ftest', targets: [{ format: 'r8unorm' }] },
    });

    const dsTexture = t.device.createTexture({
      format,
      size: [kNumDepthValues],
      usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC,
      sampleCount: multisampled ? 4 : 1,
    });
    const dsTextureView = dsTexture.createView();

    const testTextureDesc = {
      format: 'r8unorm',
      size: [kNumDepthValues],
      usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC,
    };
    const testTexture = t.device.createTexture(testTextureDesc);
    const testTextureView = testTexture.createView();
    const testTextureMSView = multisampled
      ? t.device.createTexture({ ...testTextureDesc, sampleCount: 4 }).createView()
      : undefined;

    const resultBuffer = t.device.createBuffer({
      size: kNumDepthValues,
      usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
    });

    const enc = t.device.createCommandEncoder();
    {
      const pass = enc.beginRenderPass({
        colorAttachments: [],
        depthStencilAttachment: {
          view: dsTextureView,
          depthClearValue: 1.0,
          depthLoadOp: 'clear',
          depthStoreOp: 'store',
          stencilClearValue: info.stencil ? 0 : undefined,
          stencilLoadOp: info.stencil ? 'clear' : undefined,
          stencilStoreOp: info.stencil ? 'discard' : undefined,
        },
      });
      pass.setPipeline(initPipeline);
      pass.draw(kNumDepthValues);
      pass.end();
    }
    {
      const clearValue = [0, 0, 0, 0]; // Will see this color if the test passed.
      const pass = enc.beginRenderPass({
        colorAttachments: [
          testTextureMSView
            ? {
                view: testTextureMSView,
                resolveTarget: testTextureView,
                clearValue,
                loadOp: 'clear',
                storeOp: 'discard',
              }
            : { view: testTextureView, clearValue, loadOp: 'clear', storeOp: 'store' },
        ],

        depthStencilAttachment: {
          view: dsTextureView,
          depthLoadOp: 'load',
          depthStoreOp: 'store',
          stencilClearValue: info.stencil ? 0 : undefined,
          stencilLoadOp: info.stencil ? 'clear' : undefined,
          stencilStoreOp: info.stencil ? 'discard' : undefined,
        },
      });
      pass.setPipeline(testPipeline);
      pass.setViewport(0, 0, kNumDepthValues, 1, kViewportMinDepth, kViewportMaxDepth);
      pass.draw(kNumDepthValues);
      pass.end();
    }
    enc.copyTextureToBuffer({ texture: testTexture }, { buffer: resultBuffer }, [kNumDepthValues]);
    t.device.queue.submit([enc.finish()]);

    t.expectGPUBufferValuesEqual(resultBuffer, new Uint8Array(kNumDepthValues), 0, {
      method: 'map',
    });
  });
