// META: title=validation tests for WebNN API createContext()
// META: global=window,dedicatedworker
// META: variant=?cpu
// META: variant=?gpu
// META: variant=?npu
// META: script=../resources/utils_validation.js

'use strict';

promise_test(async t => {
    const ml_context_options = {};
    await navigator.ml.createContext(ml_context_options);
}, 'Create context with default MLContextOptions.');

promise_test(async t => {
    const ml_context_options = { deviceType: 'cpu' };
    await navigator.ml.createContext(ml_context_options);
}, 'Create context with device type: cpu.');

promise_test(async t => {
    const ml_context_options = { deviceType: 'xpu' };
    const promise = navigator.ml.createContext(ml_context_options);
    promise_rejects_js(t, TypeError, promise);
}, 'Throw if deviceType is not a valid enum value of type MLDeviceType when creating the context.');

promise_test(async t => {
    const ml_context_options = { powerPreference: 'default' };
    await navigator.ml.createContext(ml_context_options);
}, 'Create context with power preference: default.');

promise_test(async t => {
    const ml_context_options = { powerPreference: 'high-performance' };
    await navigator.ml.createContext(ml_context_options);
}, 'Create context with power preference: high-performance.');

promise_test(async t => {
    const ml_context_options = { powerPreference: 'low-power' };
    await navigator.ml.createContext(ml_context_options);
}, 'Create context with power preference: low-power.');

promise_test(async t => {
    const ml_context_options = { powerPreference: 'auto' };
    const promise = navigator.ml.createContext(ml_context_options);
    promise_rejects_js(t, TypeError, promise);
}, 'Throw if powerPreference is not a valid enum value of type MLPowerPreference when creating the context.');

promise_test(async t => {
    const ml_context_options = { deviceType: 'cpu', powerPreference: 'high-performance' };
    await navigator.ml.createContext(ml_context_options);
}, '[createContext] Test creating context with deviceType=cpu, powerPreference=high-performance.');
