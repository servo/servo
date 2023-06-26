self.onmessage = event => {
  // Checking for a particular value, so more tests can be added in future.
  if (event.data !== 'test-shownotification') return;

  // Random number, so we can identify the notification we create.
  const random = Math.random().toString();
  const start = Date.now();

  event.waitUntil(
    self.registration.showNotification('test', {
      tag: random,
      // ?pipe=trickle(d2) delays the icon request by two seconds
      icon: 'icon.png?pipe=trickle(d2)'
    }).then(() => {
      const resolveDuration = Date.now() - start;

      return self.registration.getNotifications().then(notifications => {
        event.source.postMessage({
          type: 'notification-data',
          resolveDuration,
          // Check if our notification is in notifications
          notificationReturned: notifications.some(n => n.tag == random)
        });
      });
    })
  );
};