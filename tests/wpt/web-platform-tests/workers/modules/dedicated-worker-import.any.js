// Starts a dedicated worker for |scriptURL| and waits until the list of
// imported modules is sent from the worker. Passes if the list is equal to
// |expectedImportedModules|.
function import_test(scriptURL, expectedImportedModules, description) {
  promise_test(async () => {
    const worker = new Worker(scriptURL, { type: 'module' });
    const msg_event = await new Promise(resolve => worker.onmessage = resolve);
    assert_array_equals(msg_event.data, expectedImportedModules);
  }, description);
}

import_test('resources/static-import-worker.js',
            ['export-on-load-script.js'],
            'Static import.');

import_test('resources/nested-static-import-worker.js',
            ['export-on-static-import-script.js', 'export-on-load-script.js'],
            'Nested static import.');


import_test('resources/static-import-and-then-dynamic-import-worker.js',
            ['export-on-dynamic-import-script.js', 'export-on-load-script.js'],
            'Static import and then dynamic import.');

import_test('resources/dynamic-import-worker.js',
            ['export-on-load-script.js'],
            'Dynamic import.');

import_test('resources/nested-dynamic-import-worker.js',
            ['export-on-dynamic-import-script.js', 'export-on-load-script.js'],
            'Nested dynamic import.');

import_test('resources/dynamic-import-and-then-static-import-worker.js',
            ['export-on-static-import-script.js', 'export-on-load-script.js'],
            'Dynamic import and then static import.');

import_test('resources/eval-dynamic-import-worker.js',
            ['export-on-load-script.js'],
            'eval(import()).');
