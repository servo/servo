/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ import { getDefaultRequestAdapterOptions } from '../../../common/util/navigator_gpu.js';

export async function launchWorker() {
  const selfPath = import.meta.url;
  const selfPathDir = selfPath.substring(0, selfPath.lastIndexOf('/'));
  const workerPath = selfPathDir + '/worker.js';
  const worker = new Worker(workerPath, { type: 'module' });

  const promise = new Promise(resolve => {
    worker.addEventListener('message', ev => resolve(ev.data), { once: true });
  });
  worker.postMessage({ defaultRequestAdapterOptions: getDefaultRequestAdapterOptions() });
  return await promise;
}
