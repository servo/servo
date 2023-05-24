/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Returns the atomically loaded the value pointed to by atomic_ptr. It does not modify the object.
`;
import { makeTestGroup } from '../../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../../gpu_test.js';

export const g = makeTestGroup(GPUTest);

g.test('load')
  .specURL('https://www.w3.org/TR/WGSL/#atomic-load')
  .desc(
    `
AS is storage or workgroup
T is i32 or u32

fn atomicLoad(atomic_ptr: ptr<AS, atomic<T>, read_write>) -> T

`
  )
  .params(u => u.combine('SC', ['storage', 'uniform']).combine('T', ['i32', 'u32']))
  .unimplemented();
