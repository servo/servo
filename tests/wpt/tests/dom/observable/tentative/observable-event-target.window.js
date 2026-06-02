test(() => {
  // See https://dom.spec.whatwg.org/#dom-event-eventphase.
  const CAPTURING_PHASE = 1;
  const BUBBLING_PHASE = 3;

  // First, create a div underneath the `<body>` element. It will be the
  // dispatch target for synthetic click events.
  const target =
      document.querySelector('body').appendChild(document.createElement('div'));

  const body = document.querySelector('body');
  const captureObservable = body.when('click', {capture: true});
  const bubbleObservable = body.when('click', {capture: false});

  const results = [];
  captureObservable.subscribe(e => results.push(e.eventPhase));
  bubbleObservable.subscribe(e => results.push(e.eventPhase));

  target.dispatchEvent(new MouseEvent('click', {bubbles: true}));

  assert_array_equals(results, [CAPTURING_PHASE, BUBBLING_PHASE]);
}, "EventTarget Observables can listen for events in the capturing or bubbling phase");

test(() => {
  const target = new EventTarget();

  const observable = target.when('event', {passive: true});
  observable.subscribe(event => {
    assert_false(event.defaultPrevented);
    // Should do nothing, since `observable` is "passive".
    event.preventDefault();
    assert_false(event.defaultPrevented);
  });

  // Create a cancelable event which ordinarily would be able to have its
  // "default" prevented.
  const event = new Event('event', {cancelable: true});
  target.dispatchEvent(event);
}, "EventTarget Observables can be 'passive'");
