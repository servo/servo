/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export const description = `
copy{Buffer,Texture}To{Buffer,Texture} tests.
`;
import { TestGroup } from '../../../framework/index.js';
import { GPUTest } from '../gpu_test.js';
export const g = new TestGroup(GPUTest);
g.test('b2b', async t => {
  const data = new Uint32Array([0x01020304]);
  const [src, map] = t.device.createBufferMapped({
    size: 4,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  });
  new Uint32Array(map).set(data);
  src.unmap();
  const dst = t.device.createBuffer({
    size: 4,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  });
  const encoder = t.device.createCommandEncoder();
  encoder.copyBufferToBuffer(src, 0, dst, 0, 4);
  t.device.defaultQueue.submit([encoder.finish()]);
  t.expectContents(dst, data);
});
g.test('b2t2b', async t => {
  const data = new Uint32Array([0x01020304]);
  const [src, map] = t.device.createBufferMapped({
    size: 4,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  });
  new Uint32Array(map).set(data);
  src.unmap();
  const dst = t.device.createBuffer({
    size: 4,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  });
  const mid = t.device.createTexture({
    size: {
      width: 1,
      height: 1,
      depth: 1
    },
    format: 'rgba8uint',
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST
  });
  const encoder = t.device.createCommandEncoder();
  encoder.copyBufferToTexture({
    buffer: src,
    rowPitch: 256,
    imageHeight: 1
  }, {
    texture: mid,
    mipLevel: 0,
    origin: {
      x: 0,
      y: 0,
      z: 0
    }
  }, {
    width: 1,
    height: 1,
    depth: 1
  });
  encoder.copyTextureToBuffer({
    texture: mid,
    mipLevel: 0,
    origin: {
      x: 0,
      y: 0,
      z: 0
    }
  }, {
    buffer: dst,
    rowPitch: 256,
    imageHeight: 1
  }, {
    width: 1,
    height: 1,
    depth: 1
  });
  t.device.defaultQueue.submit([encoder.finish()]);
  t.expectContents(dst, data);
});
g.test('b2t2t2b', async t => {
  const data = new Uint32Array([0x01020304]);
  const [src, map] = t.device.createBufferMapped({
    size: 4,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  });
  new Uint32Array(map).set(data);
  src.unmap();
  const dst = t.device.createBuffer({
    size: 4,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  });
  const midDesc = {
    size: {
      width: 1,
      height: 1,
      depth: 1
    },
    format: 'rgba8uint',
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.COPY_DST
  };
  const mid1 = t.device.createTexture(midDesc);
  const mid2 = t.device.createTexture(midDesc);
  const encoder = t.device.createCommandEncoder();
  encoder.copyBufferToTexture({
    buffer: src,
    rowPitch: 256,
    imageHeight: 1
  }, {
    texture: mid1,
    mipLevel: 0,
    origin: {
      x: 0,
      y: 0,
      z: 0
    }
  }, {
    width: 1,
    height: 1,
    depth: 1
  });
  encoder.copyTextureToTexture({
    texture: mid1,
    mipLevel: 0,
    origin: {
      x: 0,
      y: 0,
      z: 0
    }
  }, {
    texture: mid2,
    mipLevel: 0,
    origin: {
      x: 0,
      y: 0,
      z: 0
    }
  }, {
    width: 1,
    height: 1,
    depth: 1
  });
  encoder.copyTextureToBuffer({
    texture: mid2,
    mipLevel: 0,
    origin: {
      x: 0,
      y: 0,
      z: 0
    }
  }, {
    buffer: dst,
    rowPitch: 256,
    imageHeight: 1
  }, {
    width: 1,
    height: 1,
    depth: 1
  });
  t.device.defaultQueue.submit([encoder.finish()]);
  t.expectContents(dst, data);
});
//# sourceMappingURL=copies.spec.js.map