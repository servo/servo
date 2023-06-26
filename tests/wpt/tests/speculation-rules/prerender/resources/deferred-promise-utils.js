/**
 * This file co-works with a html file and utils.js to test a promise that
 * should be deferred during prerendering.
 *
 * Usage example:
 *  Suppose the html is "prerender-promise-test.html"
 *  On prerendering page, prerender-promise-test.html?prerendering:
 *    const prerenderEventCollector = new PrerenderEventCollector();
 *    const promise = {a promise that should be deferred during prerendering};
 *    prerenderEventCollector.start(promise, {promise name});
 *
 *  On the initiator page, prerender-promise-test.html:
 *   execute
 *    `loadInitiatorPage();`
 */

// Collects events that happen relevant to a prerendering page.
// An event is added when:
// 1. start() is called.
// 2. a prerenderingchange event is dispatched on this document.
// 3. the promise passed to start() is resolved.
// 4. addEvent() is called manually.
class PrerenderEventCollector {
  constructor() {
    this.eventsSeen_ = [];
    new PrerenderChannel('close').addEventListener('message', () => {
      window.close();
    });
  }

  // Adds an event to `eventsSeen_` along with the prerendering state of the
  // page.
  addEvent(eventMessage) {
    this.eventsSeen_.push(
        {event: eventMessage, prerendering: document.prerendering});
  }

  // Starts collecting events until the promise resolves. Triggers activation by
  // telling the initiator page that it is ready for activation.
  async start(promise, promiseName) {
    assert_true(document.prerendering);
    this.addEvent(`started waiting ${promiseName}`);
    promise
        .then(
            () => {
              this.addEvent(`finished waiting ${promiseName}`);
            },
            (error) => {
              if (error instanceof Error)
                error = error.name;
              this.addEvent(`${promiseName} rejected: ${error}`);
            })
        .finally(() => {
          // Used to communicate with the main test page.
          const testChannel = new PrerenderChannel('test-channel');
          // Send the observed events back to the main test page.
          testChannel.postMessage(this.eventsSeen_);
          testChannel.close();
        });
    document.addEventListener('prerenderingchange', () => {
      this.addEvent('prerendering change');
    });

    // Post a task to give the implementation a chance to fail in case it
    // resolves a promise without waiting for activation.
    setTimeout(() => {
      // Used to communicate with the initiator page.
      const prerenderChannel = new PrerenderChannel('prerender-channel');
      // Inform the initiator page that this page is ready to be activated.
      prerenderChannel.postMessage('readyToActivate');
      prerenderChannel.close();
    }, 0);
  }
}
