test(() => {
  const target = new EventTarget();
  assert_implements(target.on, "The EventTarget interface has an `on` method");
  assert_equals(typeof target.on, "function",
      "EventTarget should have the on method");

  const testEvents = target.on("test");
  assert_true(testEvents instanceof Observable,
      "EventTarget.on returns an Observable");

  const results = [];
  testEvents.subscribe({
    next: value => results.push(value),
    error: () => results.push("error"),
    complete: () => results.push("complete"),
  });

  assert_array_equals(results, [],
      "Observable does not emit events until event is fired");

  const event = new Event("test");
  target.dispatchEvent(event);
  assert_array_equals(results, [event]);

  target.dispatchEvent(event);
  assert_array_equals(results, [event, event]);
}, "EventTarget.on() returns an Observable");

test(() => {
  const target = new EventTarget();
  const testEvents = target.on("test");
  const ac = new AbortController();
  const results = [];
  testEvents.subscribe({
    next: (value) => results.push(value),
    error: () => results.push('error'),
    complete: () => results.complete('complete'),
  }, { signal: ac.signal });

  assert_array_equals(results, [],
      "Observable does not emit events until event is fired");

  const event1 = new Event("test");
  const event2 = new Event("test");
  const event3 = new Event("test");
  target.dispatchEvent(event1);
  target.dispatchEvent(event2);

  assert_array_equals(results, [event1, event2]);

  ac.abort();
  target.dispatchEvent(event3);

  assert_array_equals(results, [event1, event2],
      "Aborting the subscription removes the event listener and stops the " +
      "emission of events");
}, "Aborting the subscription should stop the emission of events");

test(() => {
  const target = new EventTarget();
  const testEvents = target.on("test");
  const results = [];
  testEvents.subscribe(e => results.push(e));
  testEvents.subscribe(e => results.push(e));

  const event1 = new Event("test");
  const event2 = new Event("test");
  target.dispatchEvent(event1);
  target.dispatchEvent(event2);
  assert_array_equals(results, [event1, event1, event2, event2]);
}, "EventTarget Observables can multicast subscriptions for event handling");
