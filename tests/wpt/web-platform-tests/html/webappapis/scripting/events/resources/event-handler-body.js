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
