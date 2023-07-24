/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Performs the following steps atomically:
 * Load the original value pointed to by atomic_ptr.
 * Compare the original value to the value v using an equality operation.
 * Store the value v only if the result of the equality comparison was true.

Returns a two member structure, where the first member, old_value, is the original
value of the atomic object and the second member, exchanged, is whether or not
the comparison succeeded.

Note: the equality comparison may spuriously fail on some implementations.
That is, the second component of the result vector may be false even if the first
component of the result vector equals cmp.
`;
import { makeTestGroup } from '../../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../../gpu_test.js';

export const g = makeTestGroup(GPUTest);

g.test('exchange')
  .specURL('https://www.w3.org/TR/WGSL/#atomic-rmw')
  .desc(
    `
AS is storage or workgroup
T is i32 or u32

fn atomicCompareExchangeWeak(atomic_ptr: ptr<AS, atomic<T>, read_write>, cmp: T, v: T) -> __atomic_compare_exchange_result<T>

struct __atomic_compare_exchange_result<T> {
  old_value : T,    // old value stored in the atomic
  exchanged : bool, // true if the exchange was done
}
`
  )
  .params(u => u.combine('SC', ['storage', 'uniform']).combine('T', ['i32', 'u32']))
  .unimplemented();
