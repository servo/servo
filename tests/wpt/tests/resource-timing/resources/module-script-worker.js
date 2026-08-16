// Module worker importer. It imports the same target module both statically (at
// instantiation) and dynamically (while running); this worker script is the
// initiator of each import. The static import is observed in the owner
// document's timeline; the dynamic import is observed here and posted back.
import './module-script-imported.js?label=worker-importer-static';

const dynamicLabel = 'worker-importer-dynamic';
const dynamicResource = './module-script-imported.js?label=' + dynamicLabel;

const observe_entry_no_timeout = entryName => new Promise(resolve => {
  new PerformanceObserver((list, observer) => {
    for (const entry of list.getEntries()) {
      if (entry.name.endsWith(entryName)) {
        resolve(entry);
        observer.disconnect();
        return;
      }
    }
  }).observe({type: 'resource', buffered: true});
});

self.onmessage = async () => {
  await import(dynamicResource);
  const entry = await observe_entry_no_timeout(
      'module-script-imported.js?label=' + dynamicLabel);
  postMessage({result: entry.initiatorUrl, expected: self.location.href});
};
