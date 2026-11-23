export async function profile(sample = false) {
  const profiler = new Profiler({sampleInterval: 10, maxBufferSize: 100});
  if (sample) {
    for (const deadline = performance.now() + 510;
         performance.now() < deadline;) {
    }
  }
  const trace = await profiler.stop();
  return {success: true, stopped: profiler.stopped, trace};
}

const params = new URL(self.location.href).searchParams;
if (import.meta.url === self.location.href) {
  try {
    let result;
    if (params.has('import')) {
      const imported = await import(params.get('import'));
      result = await imported.profile();
    } else if (params.has('nested')) {
      const source = `
        import {profile} from ${JSON.stringify(import.meta.url)};
        profile().then(result => self.postMessage(result)).catch(error => {
          self.postMessage(
              {success: false, error: error.name, message: error.message});
        });
      `;
      const url =
          URL.createObjectURL(new Blob([source], {type: 'text/javascript'}));
      const worker = new Worker(url, {type: 'module'});
      try {
        result = await new Promise((resolve, reject) => {
          worker.onmessage = event => resolve(event.data);
          worker.onerror = event => reject(new Error(event.message));
        });
      } finally {
        worker.terminate();
        URL.revokeObjectURL(url);
      }
    } else {
      if (params.has('gc')) {
        await import('/common/gc.js');
        new Profiler({sampleInterval: 10, maxBufferSize: 100});
        await self.garbageCollect();
        await new Promise(resolve => {
          const channel = new MessageChannel();
          channel.port1.onmessage = () => {
            channel.port1.close();
            channel.port2.close();
            resolve();
          };
          channel.port2.postMessage(null);
        });
      }
      result = await profile(params.has('sample'));
    }
    self.postMessage(result);
  } catch (error) {
    self.postMessage(
        {success: false, error: error.name, message: error.message});
  }
}
