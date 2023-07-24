/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Atomically stores the value v in the atomic object pointed to by atomic_ptr.
`;
import { makeTestGroup } from '../../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../../gpu_test.js';

export const g = makeTestGroup(GPUTest);

g.test('store')
  .specURL('https://www.w3.org/TR/WGSL/#atomic-store')
  .desc(
    `
AS is storage or workgroup
T is i32 or u32

fn atomicStore(atomic_ptr: ptr<AS, atomic<T>, read_write>, v: T)
`
  )
  .params(u => u.combine('SC', ['storage', 'uniform']).combine('T', ['i32', 'u32']))
  .unimplemented();
