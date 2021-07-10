const windowReflectingBodyElementEventHandlerSet =
  new Set(['blur', 'error', 'focus', 'load', 'resize', 'scroll']);

function handlersInInterface(mainIDL, name) {
  return mainIDL.find(idl => idl.name === name).members.map(member => member.name.slice(2));
}

const handlersListPromise = fetch("/interfaces/html.idl").then(res => res.text()).then(htmlIDL => {
  const parsedHTMLIDL = WebIDL2.parse(htmlIDL);
  const windowEventHandlers = handlersInInterface(parsedHTMLIDL, "WindowEventHandlers");
  const globalEventHandlers = handlersInInterface(parsedHTMLIDL, "GlobalEventHandlers");
  const documentAndElementEventHandlers = handlersInInterface(parsedHTMLIDL, "DocumentAndElementEventHandlers");

  const shadowedHandlers = [
    ...windowReflectingBodyElementEventHandlerSet,
    ...windowEventHandlers
  ];
  const notShadowedHandlers = [
    ...globalEventHandlers.filter(name => !windowReflectingBodyElementEventHandlerSet.has(name)),
    ...documentAndElementEventHandlers
  ];
  return {
    shadowedHandlers,
    notShadowedHandlers
  };
});

function eventHandlerTest(shadowedHandlers, notShadowedHandlers, element) {
  const altBody = document.createElement(element);
  for (const [des, obj1, obj2, obj3, des1, des2, des3] of [
    ["document.body", document.body, altBody, window, "body", "alternative body", "window"],
    [`document.createElement("${element}")`, altBody, document.body, window, "alternative body", "body", "window"],
    ["window", window, document.body, altBody, "window", "body", "alternative body"]
  ]) {
    const f = () => 0;

    shadowedHandlers.forEach(handler => {
      const eventHandler = obj1['on' + handler];
      test(() => {
        obj1['on' + handler] = f;
        assert_equals(obj2['on' + handler], f, `${des2} should reflect`);
        assert_equals(obj3['on' + handler], f, `${des3} should reflect`);
      }, `shadowed ${handler} (${des})`);
      obj1['on' + handler] = eventHandler;
    });

    notShadowedHandlers.forEach(handler => {
      const eventHandler = obj1['on' + handler];
      test(() => {
        obj1['on' + handler] = f;
        assert_equals(obj2['on' + handler], null, `${des2} should reflect`);
        assert_equals(obj3['on' + handler], null, `${des3} should reflect`);
      }, `not shadowed ${handler} (${des})`);
      obj1['on' + handler] = eventHandler;
    });

    shadowedHandlers.forEach(handler => {
      test(() => {
        assert_equals(obj1['on' + handler], null, `${des1} should reflect changes to itself`);
        assert_equals(obj2['on' + handler], null, `${des2} should reflect`);
        assert_equals(obj3['on' + handler], null, `${des3} should reflect`);
      }, `shadowed ${handler} removal (${des})`);
    });
  }
}
