// META: global=window,worker
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://notifications.spec.whatwg.org/

idl_test(
  ['notifications'],
  ['service-workers', 'html', 'dom'],
  idl_array => {
    idl_array.add_objects({
      Notification: ['notification'],
    });
    if (self.ServiceWorkerGlobalScope) {
      idl_array.add_objects({
        NotificationEvent: ['notificationEvent'],
        ServiceWorkerGlobalScope: ['self'],
      });
    }
    self.notification = new Notification("Running idlharness.");
    if (self.ServiceWorkerGlobalScope) {
      self.notificationEvent = new NotificationEvent("type", { notification: notification });
    }
  }
);
