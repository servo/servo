/*---
description: Verify async test handling via print()
flags: [async]
---*/
Promise.resolve().then(() => {
    print('Test262:AsyncTestComplete');
});
