/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export const description = ``;
import { pbool, pcombine, poptions } from '../../../../common/framework/params.js';
import { TestGroup } from '../../../../common/framework/test_group.js';
import { MappingTest } from './mapping_test.js';
export const g = new TestGroup(MappingTest);
g.test('createBufferMapped', async t => {
  const size = t.params.size;
  const [buffer, arrayBuffer] = t.device.createBufferMapped({
    size,
    usage: GPUBufferUsage.COPY_SRC | (t.params.mappable ? GPUBufferUsage.MAP_WRITE : 0)
  });
  t.checkMapWrite(buffer, arrayBuffer, size);
}).params(pcombine(poptions('size', [12, 512 * 1024]), //
pbool('mappable')));
//# sourceMappingURL=create_mapped.spec.js.map