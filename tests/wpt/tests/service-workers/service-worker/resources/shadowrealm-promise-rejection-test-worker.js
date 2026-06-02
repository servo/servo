var realm = new ShadowRealm();

// Promise rejection from ShadowRealm should be handled within service
// worker thread.
realm.evaluate('Promise.reject("foo"); () => {}');

// Nested ShadowRealms are also possible.
realm.evaluate(`
const innerRealm = new ShadowRealm();
innerRealm.evaluate('Promise.reject("foo"); () => {}');
`);
