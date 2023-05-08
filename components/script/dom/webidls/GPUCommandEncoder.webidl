/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://gpuweb.github.io/gpuweb/#gpucommandencoder
[Exposed=(Window, DedicatedWorker), Serializable, Pref="dom.webgpu.enabled"]
interface GPUCommandEncoder {
    [NewObject]
    GPUComputePassEncoder beginComputePass(optional GPUComputePassDescriptor descriptor = {});
    [NewObject]
    GPURenderPassEncoder beginRenderPass(GPURenderPassDescriptor descriptor);

    undefined copyBufferToBuffer(
        GPUBuffer source,
        GPUSize64 sourceOffset,
        GPUBuffer destination,
        GPUSize64 destinationOffset,
        GPUSize64 size);

    undefined copyBufferToTexture(
        GPUImageCopyBuffer source,
        GPUImageCopyTexture destination,
        GPUExtent3D copySize);

    undefined copyTextureToBuffer(
        GPUImageCopyTexture source,
        GPUImageCopyBuffer destination,
        GPUExtent3D copySize);

    undefined copyTextureToTexture(
        GPUImageCopyTexture source,
        GPUImageCopyTexture destination,
        GPUExtent3D copySize);

    /*
    undefined copyImageBitmapToTexture(
        GPUImageBitmapCopyView source,
        GPUImageCopyTexture destination,
        GPUExtent3D copySize);
    */

    //undefined pushDebugGroup(USVString groupLabel);
    //undefined popDebugGroup();
    //undefined insertDebugMarker(USVString markerLabel);

    [NewObject]
    GPUCommandBuffer finish(optional GPUCommandBufferDescriptor descriptor = {});
};
GPUCommandEncoder includes GPUObjectBase;

dictionary GPUImageDataLayout {
    GPUSize64 offset = 0;
    GPUSize32 bytesPerRow;
    GPUSize32 rowsPerImage;
};

dictionary GPUImageCopyBuffer : GPUImageDataLayout {
    required GPUBuffer buffer;
};

dictionary GPUImageCopyExternalImage {
    required (ImageBitmap or HTMLCanvasElement or OffscreenCanvas) source;
    GPUOrigin2D origin = {};
    boolean flipY = false;
};

dictionary GPUImageCopyTexture {
    required GPUTexture texture;
    GPUIntegerCoordinate mipLevel = 0;
    GPUOrigin3D origin;
    GPUTextureAspect aspect = "all";
};

dictionary GPUImageCopyTextureTagged : GPUImageCopyTexture {
    //GPUPredefinedColorSpace colorSpace = "srgb"; //TODO
    boolean premultipliedAlpha = false;
};

dictionary GPUImageBitmapCopyView {
    //required ImageBitmap imageBitmap; //TODO
    GPUOrigin2D origin;
};

dictionary GPUComputePassDescriptor : GPUObjectDescriptorBase {
};

//
dictionary GPURenderPassDescriptor : GPUObjectDescriptorBase {
    required sequence<GPURenderPassColorAttachment> colorAttachments;
    GPURenderPassDepthStencilAttachment depthStencilAttachment;
    GPUQuerySet occlusionQuerySet;
};

dictionary GPURenderPassColorAttachment {
    required GPUTextureView view;
    GPUTextureView resolveTarget;

    GPUColor clearValue;
    required GPULoadOp loadOp;
    required GPUStoreOp storeOp;
};

dictionary GPURenderPassDepthStencilAttachment {
    required GPUTextureView view;

    float depthClearValue;
    GPULoadOp depthLoadOp;
    GPUStoreOp depthStoreOp;
    boolean depthReadOnly = false;

    GPUStencilValue stencilClearValue = 0;
    GPULoadOp stencilLoadOp;
    GPUStoreOp stencilStoreOp;
    boolean stencilReadOnly = false;
};

enum GPULoadOp {
    "load",
    "clear"
};

enum GPUStoreOp {
    "store",
    "discard"
};

dictionary GPURenderPassLayout: GPUObjectDescriptorBase {
    required sequence<GPUTextureFormat> colorFormats;
    GPUTextureFormat depthStencilFormat;
    GPUSize32 sampleCount = 1;
};

dictionary GPUColorDict {
    required double r;
    required double g;
    required double b;
    required double a;
};
typedef (sequence<double> or GPUColorDict) GPUColor;

dictionary GPUOrigin2DDict {
    GPUIntegerCoordinate x = 0;
    GPUIntegerCoordinate y = 0;
};
typedef (sequence<GPUIntegerCoordinate> or GPUOrigin2DDict) GPUOrigin2D;

dictionary GPUOrigin3DDict {
    GPUIntegerCoordinate x = 0;
    GPUIntegerCoordinate y = 0;
    GPUIntegerCoordinate z = 0;
};
typedef (sequence<GPUIntegerCoordinate> or GPUOrigin3DDict) GPUOrigin3D;
