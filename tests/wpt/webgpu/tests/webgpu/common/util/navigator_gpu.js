/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/ /// <reference types="@webgpu/types" />

import { ErrorWithExtra, assert, objectEquals } from './util.js';

/**
 * Finds and returns the `navigator.gpu` object (or equivalent, for non-browser implementations).
 * Throws an exception if not found.
 */
function defaultGPUProvider() {
  assert(
    typeof navigator !== 'undefined' && navigator.gpu !== undefined,
    'No WebGPU implementation found'
  );
  return navigator.gpu;
}

/**
 * GPUProvider is a function that creates and returns a new GPU instance.
 * May throw an exception if a GPU cannot be created.
 */


let gpuProvider = defaultGPUProvider;

/**
 * Sets the function to create and return a new GPU instance.
 */
export function setGPUProvider(provider) {
  assert(impl === undefined, 'setGPUProvider() should not be after getGPU()');
  gpuProvider = provider;
}

let impl = undefined;

let defaultRequestAdapterOptions;

export function setDefaultRequestAdapterOptions(options) {
  // It's okay to call this if you don't change the options
  if (objectEquals(options, defaultRequestAdapterOptions)) {
    return;
  }
  if (impl) {
    throw new Error('must call setDefaultRequestAdapterOptions before getGPU');
  }
  defaultRequestAdapterOptions = { ...options };
}

export function getDefaultRequestAdapterOptions() {
  return defaultRequestAdapterOptions;
}

/**
 * Finds and returns the `navigator.gpu` object (or equivalent, for non-browser implementations).
 * Throws an exception if not found.
 */
export function getGPU(recorder) {
  if (impl) {
    return impl;
  }

  impl = gpuProvider();

  if (defaultRequestAdapterOptions) {

    const oldFn = impl.requestAdapter;
    impl.requestAdapter = function (
    options)
    {
      const promise = oldFn.call(this, { ...defaultRequestAdapterOptions, ...options });
      if (recorder) {
        void promise.then(async (adapter) => {
          if (adapter) {
            const info = await adapter.requestAdapterInfo();
            const infoString = `Adapter: ${info.vendor} / ${info.architecture} / ${info.device}`;
            recorder.debug(new ErrorWithExtra(infoString, () => ({ adapterInfo: info })));
          }
        });
      }
      return promise;
    };
  }

  return impl;
}