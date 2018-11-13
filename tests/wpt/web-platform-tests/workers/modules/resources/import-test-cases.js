const testCases = [
    {
        scriptURL: 'resources/static-import-worker.js',
        expectation: ['export-on-load-script.js'],
        description: 'Static import.'
    },
    {
        scriptURL: 'resources/nested-static-import-worker.js',
        expectation: [
            'export-on-static-import-script.js',
            'export-on-load-script.js'
        ],
        description: 'Nested static import.'
    },
    {
        scriptURL: 'resources/static-import-and-then-dynamic-import-worker.js',
        expectation: [
            'export-on-dynamic-import-script.js',
            'export-on-load-script.js'
        ],
        description: 'Static import and then dynamic import.'
    },
    {
        scriptURL: 'resources/dynamic-import-worker.js',
        expectation: ['export-on-load-script.js'],
        description: 'Dynamic import.'
    },
    {
        scriptURL: 'resources/nested-dynamic-import-worker.js',
        expectation: [
            'export-on-dynamic-import-script.js',
            'export-on-load-script.js'
        ],
        description: 'Nested dynamic import.'
    },
    {
        scriptURL: 'resources/dynamic-import-and-then-static-import-worker.js',
        expectation: [
            'export-on-static-import-script.js',
            'export-on-load-script.js'
        ],
        description: 'Dynamic import and then static import.'
    },
    {
        scriptURL: 'resources/eval-dynamic-import-worker.js',
        expectation: ['export-on-load-script.js'],
        description: 'eval(import()).'
    }
];
