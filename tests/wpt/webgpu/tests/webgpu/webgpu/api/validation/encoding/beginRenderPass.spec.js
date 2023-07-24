/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Note: render pass 'occlusionQuerySet' validation is tested in queries/general.spec.ts

TODO: Check that depth-stencil attachment views must encompass all aspects.

TODO: check for duplication (render_pass/, etc.), plan, and implement.
Note possibly a lot of this should be operation tests instead.
Notes:
> - color attachments {zero, one, multiple}
>     - many different formats (some are non-renderable)
>     - is a view on a texture with multiple mip levels or array layers
>     - two attachments use the same view, or views of {intersecting, disjoint} ranges
>     - {without, with} resolve target
>         - resolve format compatibility with multisampled format
>     - {all possible load ops, load color {in range, negative, too large}}
>     - all possible store ops
> - depth/stencil attachment
>     - {unset, all possible formats}
>     - {all possible {depth, stencil} load ops, load values {in range, negative, too large}}
>     - all possible {depth, stencil} store ops
>     - depthReadOnly {t,f}, stencilReadOnly {t,f}
`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { ValidationTest } from '../validation_test.js';

export const g = makeTestGroup(ValidationTest);

g.test('color_attachments,device_mismatch')
  .desc(
    `
    Tests beginRenderPass cannot be called with color attachments whose texture view or resolve target is created from another device
    The 'view' and 'resolveTarget' are:
    - created from same device in ColorAttachment0 and ColorAttachment1
    - created from different device in ColorAttachment0 and ColorAttachment1
    - created from same device in ColorAttachment0, but from different device in ColorAttachment1
    `
  )
  .paramsSubcasesOnly([
    {
      view0Mismatched: false,
      target0Mismatched: false,
      view1Mismatched: false,
      target1Mismatched: false,
    }, // control case
    {
      view0Mismatched: false,
      target0Mismatched: true,
      view1Mismatched: false,
      target1Mismatched: true,
    },
    {
      view0Mismatched: true,
      target0Mismatched: false,
      view1Mismatched: true,
      target1Mismatched: false,
    },
    {
      view0Mismatched: false,
      target0Mismatched: false,
      view1Mismatched: false,
      target1Mismatched: true,
    },
  ])
  .beforeAllSubcases(t => {
    t.selectMismatchedDeviceOrSkipTestCase(undefined);
  })
  .fn(t => {
    const { view0Mismatched, target0Mismatched, view1Mismatched, target1Mismatched } = t.params;
    const mismatched = view0Mismatched || target0Mismatched || view1Mismatched || target1Mismatched;

    const view0Texture = view0Mismatched
      ? t.getDeviceMismatchedRenderTexture(4)
      : t.getRenderTexture(4);
    const target0Texture = target0Mismatched
      ? t.getDeviceMismatchedRenderTexture()
      : t.getRenderTexture();
    const view1Texture = view1Mismatched
      ? t.getDeviceMismatchedRenderTexture(4)
      : t.getRenderTexture(4);
    const target1Texture = target1Mismatched
      ? t.getDeviceMismatchedRenderTexture()
      : t.getRenderTexture();

    const encoder = t.createEncoder('non-pass');
    const pass = encoder.encoder.beginRenderPass({
      colorAttachments: [
        {
          view: view0Texture.createView(),
          clearValue: { r: 1.0, g: 0.0, b: 0.0, a: 1.0 },
          loadOp: 'clear',
          storeOp: 'store',
          resolveTarget: target0Texture.createView(),
        },
        {
          view: view1Texture.createView(),
          clearValue: { r: 1.0, g: 0.0, b: 0.0, a: 1.0 },
          loadOp: 'clear',
          storeOp: 'store',
          resolveTarget: target1Texture.createView(),
        },
      ],
    });
    pass.end();

    encoder.validateFinish(!mismatched);
  });

g.test('depth_stencil_attachment,device_mismatch')
  .desc(
    'Tests beginRenderPass cannot be called with a depth stencil attachment whose texture view is created from another device'
  )
  .paramsSubcasesOnly(u => u.combine('mismatched', [true, false]))
  .beforeAllSubcases(t => {
    t.selectMismatchedDeviceOrSkipTestCase(undefined);
  })
  .fn(t => {
    const { mismatched } = t.params;

    const descriptor = {
      size: { width: 4, height: 4, depthOrArrayLayers: 1 },
      format: 'depth24plus-stencil8',
      usage: GPUTextureUsage.RENDER_ATTACHMENT,
    };

    const depthStencilTexture = mismatched
      ? t.getDeviceMismatchedTexture(descriptor)
      : t.device.createTexture(descriptor);

    const encoder = t.createEncoder('non-pass');
    const pass = encoder.encoder.beginRenderPass({
      colorAttachments: [],
      depthStencilAttachment: {
        view: depthStencilTexture.createView(),
        depthClearValue: 0,
        depthLoadOp: 'clear',
        depthStoreOp: 'store',
        stencilClearValue: 0,
        stencilLoadOp: 'clear',
        stencilStoreOp: 'store',
      },
    });
    pass.end();

    encoder.validateFinish(!mismatched);
  });

g.test('occlusion_query_set,device_mismatch')
  .desc(
    'Tests beginRenderPass cannot be called with an occlusion query set created from another device'
  )
  .paramsSubcasesOnly(u => u.combine('mismatched', [true, false]))
  .beforeAllSubcases(t => {
    t.selectMismatchedDeviceOrSkipTestCase(undefined);
  })
  .fn(t => {
    const { mismatched } = t.params;
    const sourceDevice = mismatched ? t.mismatchedDevice : t.device;

    const occlusionQuerySet = sourceDevice.createQuerySet({
      type: 'occlusion',
      count: 1,
    });
    t.trackForCleanup(occlusionQuerySet);

    const encoder = t.createEncoder('render pass', { occlusionQuerySet });
    encoder.validateFinish(!mismatched);
  });

g.test('timestamp_query_set,device_mismatch')
  .desc(
    `
  Tests beginRenderPass cannot be called with a timestamp query set created from another device.
  `
  )
  .paramsSubcasesOnly(u => u.combine('mismatched', [true, false]))
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase(['timestamp-query']);
    t.selectMismatchedDeviceOrSkipTestCase('timestamp-query');
  })
  .fn(t => {
    const { mismatched } = t.params;
    const sourceDevice = mismatched ? t.mismatchedDevice : t.device;

    const timestampQuerySet = sourceDevice.createQuerySet({
      type: 'timestamp',
      count: 1,
    });

    const timestampWrites = {
      querySet: timestampQuerySet,
      beginningOfPassWriteIndex: 0,
    };

    const colorTexture = t.device.createTexture({
      format: 'rgba8unorm',
      size: { width: 4, height: 4, depthOrArrayLayers: 1 },
      usage: GPUTextureUsage.RENDER_ATTACHMENT,
    });

    const encoder = t.createEncoder('non-pass');
    const pass = encoder.encoder.beginRenderPass({
      colorAttachments: [
        {
          view: colorTexture.createView(),
          loadOp: 'load',
          storeOp: 'store',
        },
      ],

      timestampWrites,
    });
    pass.end();

    encoder.validateFinish(!mismatched);
  });
