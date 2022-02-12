promise_test(() => {
  return (new Function('w', 'return import(w)'))("./import.js?Function")
    .then(module => assert_equals(module.A.from, 'alpha/import.js'));
}, 'alpha - Function');

promise_test(() => {
  return eval('import("./import.js?eval")')
    .then(module => assert_equals(module.A.from, 'alpha/import.js'));
}, 'alpha - eval');
