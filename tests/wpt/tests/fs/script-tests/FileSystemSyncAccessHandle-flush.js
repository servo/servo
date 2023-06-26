'use strict';

// This script depends on the following scripts:
//    /fs/resources/messaging-helpers.js

sync_access_handle_test(async handle => {
  await handle.flush();
}, 'Test flush on an empty file.');
