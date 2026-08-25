# Web Neural Network API Test Suite

Spec: https://www.w3.org/TR/webnn/

Repo: https://github.com/webmachinelearning/webnn

## Web IDL Tests

The `idlharness.https.any.js` test validates that the API implements
the interfaces as defined in the Web IDL in the specification. A
snapshot of the spec's Web IDL resides in `../interfaces/webnn.idl`
and is automatically daily by shared WPT infrastructure.

## Validation Tests

The tests in `validation_tests` go beyond what can be validated from
the Web IDL definitions in the spec. They ensure that parameters to
the various WebNN methods are validated according to the algorithms in
the spec, and that invalid inputs are rejected as specified. For
example, `validation_tests/matmul.https.any.js` verifies that an
exception is thrown if the rank of either of the input tensors is less
than 2, or if the input tensor shapes are not broadcastable, etc.

## Conformance Tests

The tests in `conformance_tests` exercise the various WebNN operators
and ensure that they are behaving as expected. For example,
`conformance_tests/matmul.https.any.js` verifies that multiplying
N-dimensional matrices actually produces the expected output.

## Variations

WebNN supports execution on various compute devices, e.g. CPU, GPU and
NPU. The various tests make use of `META: variant=?...` to enable the
test harness to execute the same test multiple times, with parameters
specifying each device type in turn.

NOTE: The specific API for device selection is currently under debate.
See:
https://github.com/webmachinelearning/webnn/blob/main/device-selection-explainer.md

## Globals

WebNN interfaces are exposed to both Window and Worker global contexts
via the `navigator.ml` member. These tests make use of the `.any.js`
convention and `META: global=window,dedicatedworker` to allow the test
harness to execute the same test multiple times, once for each global
context type.

Since executing a large number of tests consumes significant compute
cycles, and since unexpected behavior differences across contexts
(e.g. `window` vs. `dedicatedworker`) for individual WebNN operators
(e.g. `matmul()`) is unlikely, most operator-specific tests only
specify `global=window`.
