// META: title=test graph.devices
// META: global=window
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils.js
// META: timeout=long

'use strict';

if (navigator.ml) {
  promise_test(async () => {
    const context = await navigator.ml.createContext(contextOptions);
    const builder = new MLGraphBuilder(context);
    const a = builder.input('A', {dataType: 'float32', shape: []});
    const o = builder.add(a, a);
    const graph = await builder.build({'o': o});
    assert_greater_than(graph.devices.length, 0, 'graph.devices is empty.');
  });
} else {
  test(() => assert_implements(navigator.ml, 'missing navigator.ml'));
}
