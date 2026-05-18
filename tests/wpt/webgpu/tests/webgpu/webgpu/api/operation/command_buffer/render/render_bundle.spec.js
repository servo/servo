/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Ensure that render bundles execute.
- Test they run
- Test they can be used multiple times
  - in different passes
  - in the same pass
  - in the same executeBundles call
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { AllFeaturesMaxLimitsGPUTest } from '../../../../gpu_test.js';
import * as ttu from '../../../../texture_test_utils.js';

export const g = makeTestGroup(AllFeaturesMaxLimitsGPUTest);

// Makes a render pipeline where we can select kColor0 or kColor1 by instance index
// We can select top right triangle or bottom left triangle by vertex index
function makeRenderPipeline(device, blend = undefined) {
  const module = device.createShaderModule({
    code: `
      struct Interop {
         @builtin(position) pos: vec4f,
         @location(0) @interpolate(flat, either) inst: u32,
      }
      @vertex fn vs(@builtin(vertex_index) vNdx: u32,
                    @builtin(instance_index) inst: u32) -> Interop {
        let pos = array(
          vec2f(-1, -1), vec2f(1, -1), vec2f(-1, 1),
          vec2f(-1,  1), vec2f(1, -1), vec2f( 1, 1),
        );
        return Interop(vec4f(pos[vNdx], 0, 1), inst);
      }

      @fragment fn fs(v: Interop) -> @location(0) vec4f {
        // round these colors a little since different GPUs might go up or down a bit to rgba8unorm
        let colors = array(
          vec4f(1.1 / 255, 2.1 / 255, 3.1 / 255, 4.1 / 255),
          vec4f(5.1 / 255, 6.1 / 255, 7.1 / 255, 8.1 / 255),
        );
        return colors[v.inst];
      }
    `
  });

  const pipeline = device.createRenderPipeline({
    layout: 'auto',
    vertex: { module },
    fragment: { module, targets: [{ format: 'rgba8unorm', blend }] }
  });

  return pipeline;
}

function makeRenderPass(encoder, view) {
  const pass = encoder.beginRenderPass({
    colorAttachments: [
    {
      view,
      loadOp: 'clear',
      storeOp: 'store'
    }]

  });
  return pass;
}

function makeTexture(t) {
  return t.createTextureTracked({
    size: [4, 4],
    format: 'rgba8unorm',
    usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC
  });
}

const kColor0 = { R: 1 / 255, G: 2 / 255, B: 3 / 255, A: 4 / 255 };
const kColor1 = { R: 5 / 255, G: 6 / 255, B: 7 / 255, A: 8 / 255 };
const kColor0x3 = { R: kColor0.R * 3, G: kColor0.G * 3, B: kColor0.B * 3, A: kColor0.A * 3 };
const kZero = { R: 0, G: 0, B: 0, A: 0 };

g.test('basic').
desc(`Test a basic render bundle`).
fn((t) => {
  const pipeline = makeRenderPipeline(t.device);

  const bundleEncoder = t.device.createRenderBundleEncoder({
    colorFormats: ['rgba8unorm']
  });
  bundleEncoder.setPipeline(pipeline);
  bundleEncoder.draw(6);
  const bundle = bundleEncoder.finish();

  const texture = makeTexture(t);

  const encoder = t.device.createCommandEncoder();
  const pass = makeRenderPass(encoder, texture.createView());
  pass.executeBundles([bundle]);
  pass.end();
  t.device.queue.submit([encoder.finish()]);

  ttu.expectSingleColorWithTolerance(t, texture, 'rgba8unorm', {
    size: [4, 4, 1],
    exp: kColor0
  });
});

g.test('two_bundles').
desc(`Test drawing 2 render bundles`).
fn((t) => {
  const pipeline = makeRenderPipeline(t.device);

  const bundleEncoder1 = t.device.createRenderBundleEncoder({
    colorFormats: ['rgba8unorm']
  });
  bundleEncoder1.setPipeline(pipeline);
  bundleEncoder1.draw(3);
  const bundle1 = bundleEncoder1.finish();

  const bundleEncoder2 = t.device.createRenderBundleEncoder({
    colorFormats: ['rgba8unorm']
  });
  bundleEncoder2.setPipeline(pipeline);
  bundleEncoder2.draw(3, 1, 3, 1);
  const bundle2 = bundleEncoder2.finish();

  const texture = makeTexture(t);

  const encoder = t.device.createCommandEncoder();
  const pass = makeRenderPass(encoder, texture.createView());
  pass.executeBundles([bundle1, bundle2]);
  pass.end();
  t.device.queue.submit([encoder.finish()]);

  ttu.expectSinglePixelComparisonsAreOkInTexture(t, { texture }, [
  { coord: { x: 0, y: 3 }, exp: kColor0 },
  { coord: { x: 3, y: 0 }, exp: kColor1 }]
  );
});

g.test('one_bundle_used_multiple_times').
desc(`Test drawing 1 render bundle multiple times using the viewport to select where`).
fn((t) => {
  const pipeline = makeRenderPipeline(t.device);

  const bundleEncoder = t.device.createRenderBundleEncoder({
    colorFormats: ['rgba8unorm']
  });
  bundleEncoder.setPipeline(pipeline);
  bundleEncoder.draw(6);
  const bundle = bundleEncoder.finish();

  const texture = makeTexture(t);

  const encoder = t.device.createCommandEncoder();
  const pass = makeRenderPass(encoder, texture.createView());
  pass.setViewport(0, 0, 1, 1, 0, 1);
  pass.executeBundles([bundle]);
  pass.setViewport(2, 0, 1, 1, 0, 1);
  pass.executeBundles([bundle]);
  pass.setViewport(0, 2, 1, 1, 0, 1);
  pass.executeBundles([bundle]);
  pass.setViewport(2, 2, 1, 1, 0, 1);
  pass.executeBundles([bundle]);
  pass.end();
  t.device.queue.submit([encoder.finish()]);

  ttu.expectSinglePixelComparisonsAreOkInTexture(t, { texture }, [
  { coord: { x: 0, y: 0 }, exp: kColor0 },
  { coord: { x: 2, y: 0 }, exp: kColor0 },
  { coord: { x: 0, y: 2 }, exp: kColor0 },
  { coord: { x: 2, y: 2 }, exp: kColor0 },
  // Check a few places we should not have rendered.
  { coord: { x: 1, y: 0 }, exp: kZero },
  { coord: { x: 3, y: 0 }, exp: kZero },
  { coord: { x: 0, y: 1 }, exp: kZero },
  { coord: { x: 0, y: 3 }, exp: kZero },
  { coord: { x: 3, y: 3 }, exp: kZero }]
  );
});

g.test('one_bundle_used_multiple_times_same_executeBundles').
desc(`Test drawing 1 render bundle multiple times using the same call to executeBundles`).
fn((t) => {
  const pipeline = makeRenderPipeline(t.device, {
    color: { srcFactor: 'one', dstFactor: 'one', operation: 'add' },
    alpha: { srcFactor: 'one', dstFactor: 'one', operation: 'add' }
  });

  const bundleEncoder = t.device.createRenderBundleEncoder({
    colorFormats: ['rgba8unorm']
  });
  bundleEncoder.setPipeline(pipeline);
  bundleEncoder.draw(6);
  const bundle = bundleEncoder.finish();

  const texture = makeTexture(t);

  const encoder = t.device.createCommandEncoder();
  const pass = makeRenderPass(encoder, texture.createView());
  pass.executeBundles([bundle, bundle, bundle]);
  pass.end();
  t.device.queue.submit([encoder.finish()]);

  // Check the result is kColor0 added 3 times.
  ttu.expectSingleColorWithTolerance(t, texture, 'rgba8unorm', {
    size: [4, 4, 1],
    exp: kColor0x3
  });
});