/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Test that workgroup size is set correctly`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { iterRange } from '../../../../common/util/util.js';
import { GPUTest } from '../../../gpu_test.js';

export const g = makeTestGroup(GPUTest);

function checkResults(
sizeX,
sizeY,
sizeZ,
numWGs,
data)
{
  const totalInvocations = sizeX * sizeY * sizeZ;
  for (let i = 0; i < numWGs; i++) {
    const wgx_data = data[4 * i + 0];
    const wgy_data = data[4 * i + 1];
    const wgz_data = data[4 * i + 2];
    const total_data = data[4 * i + 3];
    if (wgx_data !== sizeX) {
      let msg = `Incorrect workgroup size x dimension for wg ${i}:\n`;
      msg += `- expected: ${wgx_data}\n`;
      msg += `- got:      ${sizeX}`;
      return Error(msg);
    }
    if (wgy_data !== sizeY) {
      let msg = `Incorrect workgroup size y dimension for wg ${i}:\n`;
      msg += `- expected: ${wgy_data}\n`;
      msg += `- got:      ${sizeY}`;
      return Error(msg);
    }
    if (wgz_data !== sizeZ) {
      let msg = `Incorrect workgroup size y dimension for wg ${i}:\n`;
      msg += `- expected: ${wgz_data}\n`;
      msg += `- got:      ${sizeZ}`;
      return Error(msg);
    }
    if (total_data !== totalInvocations) {
      let msg = `Incorrect workgroup total invocations for wg ${i}:\n`;
      msg += `- expected: ${total_data}\n`;
      msg += `- got:      ${totalInvocations}`;
      return Error(msg);
    }
  }
  return undefined;
}

g.test('workgroup_size').
desc(`Test workgroup size is set correctly`).
params((u) =>
u.
combine('wgx', [1, 3, 4, 8, 11, 16, 51, 64, 128, 256]).
combine('wgy', [1, 3, 4, 8, 16, 51, 64, 256]).
combine('wgz', [1, 3, 11, 16, 128, 256]).
beginSubcases()
).
fn(async (t) => {
  const {
    maxComputeWorkgroupSizeX,
    maxComputeWorkgroupSizeY,
    maxComputeWorkgroupSizeZ,
    maxComputeInvocationsPerWorkgroup
  } = t.device.limits;
  t.skipIf(
    t.params.wgx > maxComputeWorkgroupSizeX,
    `workgroup size x: ${t.params.wgx} > limit: ${maxComputeWorkgroupSizeX}`
  );
  t.skipIf(
    t.params.wgy > maxComputeWorkgroupSizeY,
    `workgroup size x: ${t.params.wgy} > limit: ${maxComputeWorkgroupSizeY}`
  );
  t.skipIf(
    t.params.wgz > maxComputeWorkgroupSizeZ,
    `workgroup size x: ${t.params.wgz} > limit: ${maxComputeWorkgroupSizeZ}`
  );
  const totalInvocations = t.params.wgx * t.params.wgy * t.params.wgz;
  t.skipIf(
    totalInvocations > maxComputeInvocationsPerWorkgroup,
    `workgroup size: ${totalInvocations} > limit: ${maxComputeInvocationsPerWorkgroup}`
  );

  const code = `
struct Values {
  x : atomic<u32>,
  y : atomic<u32>,
  z : atomic<u32>,
  total : atomic<u32>,
}

@group(0) @binding(0)
var<storage, read_write> v : array<Values>;

@compute @workgroup_size(${t.params.wgx}, ${t.params.wgy}, ${t.params.wgz})
fn main(@builtin(local_invocation_id) lid : vec3u,
        @builtin(workgroup_id) wgid : vec3u) {
  atomicMax(&v[wgid.x].x, lid.x + 1);
  atomicMax(&v[wgid.x].y, lid.y + 1);
  atomicMax(&v[wgid.x].z, lid.z + 1);
  atomicAdd(&v[wgid.x].total, 1);
}`;

  const pipeline = t.device.createComputePipeline({
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({
        code
      }),
      entryPoint: 'main'
    }
  });

  const numWorkgroups = totalInvocations < 256 ? 5 : 3;
  const buffer = t.makeBufferWithContents(
    new Uint32Array([...iterRange(numWorkgroups * 4, (_i) => 0)]),
    GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  );
  t.trackForCleanup(buffer);

  const bg = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [
    {
      binding: 0,
      resource: {
        buffer
      }
    }]

  });

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, bg);
  pass.dispatchWorkgroups(numWorkgroups, 1, 1);
  pass.end();
  t.queue.submit([encoder.finish()]);

  const bufferReadback = await t.readGPUBufferRangeTyped(buffer, {
    srcByteOffset: 0,
    type: Uint32Array,
    typedLength: 4 * numWorkgroups,
    method: 'copy'
  });
  const data = bufferReadback.data;

  t.expectOK(checkResults(t.params.wgx, t.params.wgy, t.params.wgz, numWorkgroups, data));
});