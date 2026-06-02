/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export function createQuerySetWithType(
t,
type,
count)
{
  return t.createQuerySetTracked({
    type,
    count
  });
}

export function beginRenderPassWithQuerySet(
t,
encoder,
querySet)
{
  const view = t.
  createTextureTracked({
    format: 'rgba8unorm',
    size: { width: 16, height: 16, depthOrArrayLayers: 1 },
    usage: GPUTextureUsage.RENDER_ATTACHMENT
  }).
  createView();
  return encoder.beginRenderPass({
    colorAttachments: [
    {
      view,
      clearValue: { r: 1.0, g: 0.0, b: 0.0, a: 1.0 },
      loadOp: 'clear',
      storeOp: 'store'
    }],

    occlusionQuerySet: querySet
  });
}