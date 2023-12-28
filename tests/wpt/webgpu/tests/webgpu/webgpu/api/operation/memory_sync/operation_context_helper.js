/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { assert, unreachable } from '../../../../common/util/util.js';

/**
 * Boundary between the first operation, and the second operation.
 */
export const kOperationBoundaries = [
'queue-op', // Operations are performed in different queue operations (submit, writeTexture).
'command-buffer', // Operations are in different command buffers.
'pass', // Operations are in different passes.
'execute-bundles', // Operations are in different executeBundles(...) calls
'render-bundle', // Operations are in different render bundles.
'dispatch', // Operations are in different dispatches.
'draw' // Operations are in different draws.
];


/**
 * Context a particular operation is permitted in.
 * These contexts should be sorted such that the first is the most top-level
 * context, and the last is most nested (inside a render bundle, in a render pass, ...).
 */
export const kOperationContexts = [
'queue', // Operation occurs on the GPUQueue object
'command-encoder', // Operation may be encoded in a GPUCommandEncoder.
'compute-pass-encoder', // Operation may be encoded in a GPUComputePassEncoder.
'render-pass-encoder', // Operation may be encoded in a GPURenderPassEncoder.
'render-bundle-encoder' // Operation may be encoded in a GPURenderBundleEncoder.
];







function combineContexts(
as,
bs)
{
  const result = [];
  for (const a of as) {
    for (const b of bs) {
      result.push([a, b]);
    }
  }
  return result;
}

const queueContexts = combineContexts(kOperationContexts, kOperationContexts);
const commandBufferContexts = combineContexts(
  kOperationContexts.filter((c) => c !== 'queue'),
  kOperationContexts.filter((c) => c !== 'queue')
);

/**
 * Mapping of OperationBoundary => to a set of OperationContext pairs.
 * The boundary is capable of separating operations in those two contexts.
 */
export const kBoundaryInfo =

{
  'queue-op': {
    contexts: queueContexts
  },
  'command-buffer': {
    contexts: commandBufferContexts
  },
  pass: {
    contexts: [
    ['compute-pass-encoder', 'compute-pass-encoder'],
    ['compute-pass-encoder', 'render-pass-encoder'],
    ['render-pass-encoder', 'compute-pass-encoder'],
    ['render-pass-encoder', 'render-pass-encoder'],
    ['render-bundle-encoder', 'render-pass-encoder'],
    ['render-pass-encoder', 'render-bundle-encoder'],
    ['render-bundle-encoder', 'render-bundle-encoder']]

  },
  'execute-bundles': {
    contexts: [['render-bundle-encoder', 'render-bundle-encoder']]
  },
  'render-bundle': {
    contexts: [
    ['render-bundle-encoder', 'render-pass-encoder'],
    ['render-pass-encoder', 'render-bundle-encoder'],
    ['render-bundle-encoder', 'render-bundle-encoder']]

  },
  dispatch: {
    contexts: [['compute-pass-encoder', 'compute-pass-encoder']]
  },
  draw: {
    contexts: [
    ['render-pass-encoder', 'render-pass-encoder'],
    ['render-bundle-encoder', 'render-pass-encoder'],
    ['render-pass-encoder', 'render-bundle-encoder']]

  }
};

export class OperationContextHelper {
  // We start at the queue context which is top-level.
  currentContext = 'queue';

  // Set based on the current context.









  commandBuffers = [];
  renderBundles = [];

  kTextureSize = [4, 4];
  kTextureFormat = 'rgba8unorm';

  constructor(t) {
    this.t = t;
    this.device = t.device;
    this.queue = t.device.queue;
  }

  // Ensure that all encoded commands are finished and submitted.
  ensureSubmit() {
    this.ensureContext('queue');
    this.flushCommandBuffers();
  }

  popContext() {
    switch (this.currentContext) {
      case 'queue':
        unreachable();
        break;
      case 'command-encoder':{
          assert(this.commandEncoder !== undefined);
          const commandBuffer = this.commandEncoder.finish();
          this.commandEncoder = undefined;
          this.currentContext = 'queue';
          return commandBuffer;
        }
      case 'compute-pass-encoder':
        assert(this.computePassEncoder !== undefined);
        this.computePassEncoder.end();
        this.computePassEncoder = undefined;
        this.currentContext = 'command-encoder';
        break;
      case 'render-pass-encoder':
        assert(this.renderPassEncoder !== undefined);
        this.renderPassEncoder.end();
        this.renderPassEncoder = undefined;
        this.currentContext = 'command-encoder';
        break;
      case 'render-bundle-encoder':{
          assert(this.renderBundleEncoder !== undefined);
          const renderBundle = this.renderBundleEncoder.finish();
          this.renderBundleEncoder = undefined;
          this.currentContext = 'render-pass-encoder';
          return renderBundle;
        }
    }
    return null;
  }

  makeDummyAttachment() {
    const texture = this.t.trackForCleanup(
      this.device.createTexture({
        format: this.kTextureFormat,
        size: this.kTextureSize,
        usage: GPUTextureUsage.RENDER_ATTACHMENT
      })
    );
    return {
      view: texture.createView(),
      loadOp: 'load',
      storeOp: 'store'
    };
  }

  ensureContext(context) {
    // Find the common ancestor. So we can transition from currentContext -> context.
    const ancestorContext =
    kOperationContexts[
    Math.min(
      kOperationContexts.indexOf(context),
      kOperationContexts.indexOf(this.currentContext)
    )];


    // Pop the context until we're at the common ancestor.
    while (this.currentContext !== ancestorContext) {
      // About to pop the render pass encoder. Execute any outstanding render bundles.
      if (this.currentContext === 'render-pass-encoder') {
        this.flushRenderBundles();
      }

      const result = this.popContext();
      if (result) {
        if (result instanceof GPURenderBundle) {
          this.renderBundles.push(result);
        } else {
          this.commandBuffers.push(result);
        }
      }
    }

    if (this.currentContext === context) {
      return;
    }

    switch (context) {
      case 'queue':
        unreachable();
        break;
      case 'command-encoder':
        assert(this.currentContext === 'queue');
        this.commandEncoder = this.device.createCommandEncoder();
        break;
      case 'compute-pass-encoder':
        switch (this.currentContext) {
          case 'queue':
            this.commandEncoder = this.device.createCommandEncoder();
          // fallthrough
          case 'command-encoder':
            assert(this.commandEncoder !== undefined);
            this.computePassEncoder = this.commandEncoder.beginComputePass();
            break;
          case 'compute-pass-encoder':
          case 'render-bundle-encoder':
          case 'render-pass-encoder':
            unreachable();
        }
        break;
      case 'render-pass-encoder':
        switch (this.currentContext) {
          case 'queue':
            this.commandEncoder = this.device.createCommandEncoder();
          // fallthrough
          case 'command-encoder':
            assert(this.commandEncoder !== undefined);
            this.renderPassEncoder = this.commandEncoder.beginRenderPass({
              colorAttachments: [this.makeDummyAttachment()]
            });
            break;
          case 'render-pass-encoder':
          case 'render-bundle-encoder':
          case 'compute-pass-encoder':
            unreachable();
        }
        break;
      case 'render-bundle-encoder':
        switch (this.currentContext) {
          case 'queue':
            this.commandEncoder = this.device.createCommandEncoder();
          // fallthrough
          case 'command-encoder':
            assert(this.commandEncoder !== undefined);
            this.renderPassEncoder = this.commandEncoder.beginRenderPass({
              colorAttachments: [this.makeDummyAttachment()]
            });
          // fallthrough
          case 'render-pass-encoder':
            this.renderBundleEncoder = this.device.createRenderBundleEncoder({
              colorFormats: [this.kTextureFormat]
            });
            break;
          case 'render-bundle-encoder':
          case 'compute-pass-encoder':
            unreachable();
        }
        break;
    }
    this.currentContext = context;
  }

  flushRenderBundles() {
    assert(this.renderPassEncoder !== undefined);
    if (this.renderBundles.length) {
      this.renderPassEncoder.executeBundles(this.renderBundles);
      this.renderBundles = [];
    }
  }

  flushCommandBuffers() {
    if (this.commandBuffers.length) {
      this.queue.submit(this.commandBuffers);
      this.commandBuffers = [];
    }
  }

  ensureBoundary(boundary) {
    switch (boundary) {
      case 'command-buffer':
        this.ensureContext('queue');
        break;
      case 'queue-op':
        this.ensureContext('queue');
        // Submit any GPUCommandBuffers so the next one is in a separate submit.
        this.flushCommandBuffers();
        break;
      case 'dispatch':
        // Nothing to do to separate dispatches.
        assert(this.currentContext === 'compute-pass-encoder');
        break;
      case 'draw':
        // Nothing to do to separate draws.
        assert(
          this.currentContext === 'render-pass-encoder' ||
          this.currentContext === 'render-bundle-encoder'
        );
        break;
      case 'pass':
        this.ensureContext('command-encoder');
        break;
      case 'render-bundle':
        this.ensureContext('render-pass-encoder');
        break;
      case 'execute-bundles':
        this.ensureContext('render-pass-encoder');
        // Execute any GPURenderBundles so the next one is in a separate executeBundles.
        this.flushRenderBundles();
        break;
    }
  }
}