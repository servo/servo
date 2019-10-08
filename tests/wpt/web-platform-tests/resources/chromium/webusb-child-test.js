'use strict';

// This polyfill prepares a child context to be attached to a parent context.
// The parent must call navigator.usb.test.attachToContext() to attach to the
// child context.
(() => {
  if (this.constructor.name === 'DedicatedWorkerGlobalScope' ||
      this !== window.top) {

    // Run Chromium specific set up code.
    if (typeof MojoInterfaceInterceptor !== 'undefined') {
      let messageChannel = new MessageChannel();
      messageChannel.port1.onmessage = async (messageEvent) => {
        if (messageEvent.data.type === 'Attach') {
          messageEvent.data.interfaces.forEach(interfaceName => {
            let interfaceInterceptor =
                new MojoInterfaceInterceptor(interfaceName, "context", true);
            interfaceInterceptor.oninterfacerequest =
              e => messageChannel.port1.postMessage({
                type: interfaceName,
                handle: e.handle
              }, [e.handle]);
            interfaceInterceptor.start();
          });

          // Wait for a call to GetDevices() to ensure that the interface
          // handles are forwarded to the parent context.
          await navigator.usb.getDevices();
          messageChannel.port1.postMessage({ type: 'Complete' });
        }
      };

      let message = { type: 'ReadyForAttachment', port: messageChannel.port2 };
      if (typeof Window !== 'undefined')
        parent.postMessage(message, '*', [messageChannel.port2]);
      else
        postMessage(message, [messageChannel.port2]);
    }
  }
})();
