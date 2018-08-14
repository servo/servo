// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// http://www.w3.org/TR/webstorage/#storage

idl_test(
  [], [], // Srcs + deps manually handled below.
  async idl_array => {
    const [html, dom] = await Promise.all(['html', 'dom']
        .map(i => fetch(`/interfaces/${i}.idl`).then(r => r.text())));
    idl_array.add_idls(html, {
      only: [
        'Storage',
        'WindowSessionStorage',
        'WindowLocalStorage',
        'StorageEvent',
        'StorageEventInit',
      ]});
    idl_array.add_dependency_idls(dom);

    idl_array.add_objects({
      Storage: [
        'localStorage',
        'sessionStorage',
      ],
      StorageEvent: ['new StorageEvent("storage")']
    });
  }
);
