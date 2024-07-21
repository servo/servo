/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Basic tests.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { memcpy } from '../../../../common/util/util.js';
import { GPUTest } from '../../../gpu_test.js';

export const g = makeTestGroup(GPUTest);

g.test('empty').fn((t) => {
  const encoder = t.device.createCommandEncoder();
  const cmd = encoder.finish();
  t.device.queue.submit([cmd]);
});

g.test('b2t2b').fn((t) => {
  const data = new Uint32Array([0x01020304]);

  const src = t.createBufferTracked({
    mappedAtCreation: true,
    size: 4,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  });
  memcpy({ src: data }, { dst: src.getMappedRange() });
  src.unmap();

  const dst = t.createBufferTracked({
    size: 4,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  });

  const mid = t.createTextureTracked({
    size: { width: 1, height: 1, depthOrArrayLayers: 1 },
    format: 'rgba8uint',
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST
  });

  const encoder = t.device.createCommandEncoder();
  encoder.copyBufferToTexture(
    { buffer: src, bytesPerRow: 256 },
    { texture: mid, mipLevel: 0, origin: { x: 0, y: 0, z: 0 } },
    { width: 1, height: 1, depthOrArrayLayers: 1 }
  );
  encoder.copyTextureToBuffer(
    { texture: mid, mipLevel: 0, origin: { x: 0, y: 0, z: 0 } },
    { buffer: dst, bytesPerRow: 256 },
    { width: 1, height: 1, depthOrArrayLayers: 1 }
  );
  t.device.queue.submit([encoder.finish()]);

  t.expectGPUBufferValuesEqual(dst, data);
});

g.test('b2t2t2b').fn((t) => {
  const data = new Uint32Array([0x01020304]);

  const src = t.createBufferTracked({
    mappedAtCreation: true,
    size: 4,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  });
  memcpy({ src: data }, { dst: src.getMappedRange() });
  src.unmap();

  const dst = t.createBufferTracked({
    size: 4,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  });

  const midDesc = {
    size: { width: 1, height: 1, depthOrArrayLayers: 1 },
    format: 'rgba8uint',
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST
  };
  const mid1 = t.createTextureTracked(midDesc);
  const mid2 = t.createTextureTracked(midDesc);

  const encoder = t.device.createCommandEncoder();
  encoder.copyBufferToTexture(
    { buffer: src, bytesPerRow: 256 },
    { texture: mid1, mipLevel: 0, origin: { x: 0, y: 0, z: 0 } },
    { width: 1, height: 1, depthOrArrayLayers: 1 }
  );
  encoder.copyTextureToTexture(
    { texture: mid1, mipLevel: 0, origin: { x: 0, y: 0, z: 0 } },
    { texture: mid2, mipLevel: 0, origin: { x: 0, y: 0, z: 0 } },
    { width: 1, height: 1, depthOrArrayLayers: 1 }
  );
  encoder.copyTextureToBuffer(
    { texture: mid2, mipLevel: 0, origin: { x: 0, y: 0, z: 0 } },
    { buffer: dst, bytesPerRow: 256 },
    { width: 1, height: 1, depthOrArrayLayers: 1 }
  );
  t.device.queue.submit([encoder.finish()]);

  t.expectGPUBufferValuesEqual(dst, data);
});