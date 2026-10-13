// Helper module used by exact-url-scope-immutability.html.
//
// When globalThis.doImport(specifier) is invoked, the resulting import()
// call's referring script URL is this file's URL. That URL is used as an
// exact-URL scope key in the test's second import map.
globalThis.doImport = (specifier) => import(specifier);
