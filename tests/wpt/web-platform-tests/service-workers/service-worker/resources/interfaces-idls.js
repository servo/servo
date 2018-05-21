var idls = {};
idls.untested = `
[Exposed=Worker]
interface WorkerGlobalScope {};

[TreatNonObjectAsNull]
callback EventHandlerNonNull = any (Event event);
typedef EventHandlerNonNull? EventHandler;

[NoInterfaceObject, Exposed=(Window,Worker)]
interface AbstractWorker {
  attribute EventHandler onerror;
};
`;
