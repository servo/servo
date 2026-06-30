/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// Source: Web Neural Network API (https://www.w3.org/TR/webnn/#navigatorml)

interface mixin NavigatorML {
  [SecureContext, SameObject, Pref="dom_webnn_enabled"] readonly attribute ML ml;
};
Navigator includes NavigatorML;
WorkerNavigator includes NavigatorML;

enum MLPowerPreference {
  "default",
  "high-performance",
  "low-power"
};

dictionary MLContextOptions {
  MLPowerPreference powerPreference = "default";
  boolean accelerated = true;
};

[Exposed=(Window, Worker), SecureContext, Pref="dom_webnn_enabled"]
interface ML {
  Promise<MLContext> createContext(optional MLContextOptions options = {});
};

typedef record<USVString, MLTensor> MLNamedTensors;

dictionary MLContextLostInfo {
  DOMString message;
};

[Exposed=(Window, Worker), SecureContext, Pref="dom_webnn_enabled"]
interface MLContext {
  [Throws] undefined dispatch(MLGraph graph, MLNamedTensors inputs, MLNamedTensors outputs);
  Promise<MLTensor> createTensor(MLTensorDescriptor descriptor);
  Promise<MLTensor> createConstantTensor(MLOperandDescriptor descriptor, [AllowShared] BufferSource inputData);
  Promise<ArrayBuffer> readTensor(MLTensor tensor);
  Promise<undefined> readTensor(MLTensor tensor, [AllowShared] BufferSource outputData);
  [Throws] undefined writeTensor(MLTensor tensor, [AllowShared] BufferSource inputData);
  MLOpSupportLimits opSupportLimits();
  undefined destroy();
  readonly attribute boolean accelerated;
  readonly attribute Promise<MLContextLostInfo> lost;
};

dictionary MLRankRange {
  unsigned long min;
  unsigned long max;
};

typedef sequence<MLOperandDataType> MLDataTypeList;

dictionary MLTensorLimits {
  MLDataTypeList dataTypes;
  MLRankRange rankRange;
};

dictionary MLSingleInputSupportLimits {
  MLTensorLimits input;
  MLTensorLimits output;
};

dictionary MLOpSupportLimits {
  MLInputOperandLayout preferredInputLayout;
  [EnforceRange] unsigned long long maxTensorByteLength;
  MLTensorLimits input;
  MLTensorLimits constant;
  MLTensorLimits output;
  MLSingleInputSupportLimits add;
};

[Exposed=(Window, Worker), SecureContext, Pref="dom_webnn_enabled"]
interface MLGraph {
  undefined destroy();
};

enum MLInputOperandLayout {
  "nchw",
  "nhwc"
};

enum MLOperandDataType {
  "float32",
  "float16",
  "int32",
  "uint32",
  "int64",
  "uint64",
  "int8",
  "uint8"
};

dictionary MLOperandDescriptor {
  required MLOperandDataType dataType;
  required sequence<[EnforceRange] unsigned long> shape;
};

[Exposed=(Window, Worker), SecureContext, Pref="dom_webnn_enabled"]
interface MLOperand {
  readonly attribute MLOperandDataType dataType;
  readonly attribute any shape;
};

dictionary MLOperatorOptions {
  USVString label = "";
};

dictionary MLTensorDescriptor : MLOperandDescriptor {
  boolean readable = false;
  boolean writable = false;
};

[Exposed=(Window, Worker), SecureContext, Pref="dom_webnn_enabled"]
interface MLTensor {
  readonly attribute MLOperandDataType dataType;
  readonly attribute any shape;
  readonly attribute boolean readable;
  readonly attribute boolean writable;
  readonly attribute boolean constant;
  undefined destroy();
};

typedef record<USVString, MLOperand> MLNamedOperands;

[Exposed=(Window, Worker), SecureContext, Pref="dom_webnn_enabled"]
interface MLGraphBuilder {
  [Throws] constructor(MLContext context);
  [Throws] MLOperand input(USVString name, MLOperandDescriptor descriptor);
  [Throws] MLOperand constant(MLOperandDescriptor descriptor, [AllowShared] BufferSource buffer);
  [Throws] Promise<MLGraph> build(MLNamedOperands outputs);
};

partial interface MLGraphBuilder {
  [Throws] MLOperand add(MLOperand a, MLOperand b, optional MLOperatorOptions options = {});
};
