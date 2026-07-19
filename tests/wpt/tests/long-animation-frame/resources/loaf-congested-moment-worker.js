// Worker side of the LoAF congested-moment test. A congested moment is
// surfaced as a `long-animation-frame` entry inside the worker.
importScripts('utils-worker.js');

self.onmessage = async (e) => {
  if (e.data !== 'start') {
    return;
  }

  const entry = await generate_long_animation_frame();
  self.postMessage({
    entryType: entry.entryType,
    duration: entry.duration,
    scripts: (entry.scripts ?? []).map((s) => ({
                                         invoker: s.invoker,
                                         sourceURL: s.sourceURL,
                                       })),
  });
};
