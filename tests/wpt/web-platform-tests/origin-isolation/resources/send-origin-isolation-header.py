def main(request, response):
    """Send a response with the Origin-Isolation header given in the "header"
    query parameter, or no header if that is not provided. In either case, the
    response will listen for message and messageerror events and echo them back
    to the parent. See ./helpers.mjs for how these handlers are used.
    """

    if "header" in request.GET:
      header = request.GET.first("header")
      response.headers.set("Origin-Isolation", header)

    response.headers.set("Content-Type", "text/html")

    return """
    <!DOCTYPE html>
    <meta charset="utf-8">
    <title>Helper page for origin isolation tests</title>

    <script type="module">
    import { sendWasmModule } from "./helpers.mjs";

    window.onmessage = async (e) => {
      // These could come from the parent or siblings.
      if (e.data.constructor === WebAssembly.Module) {
        e.source.postMessage("WebAssembly.Module message received", "*");
      }

      // These only come from the parent.
      if (e.data.command === "set document.domain") {
        document.domain = e.data.newDocumentDomain;
        parent.postMessage("document.domain is set", "*");
      } else if (e.data.command === "send WASM module") {
        const destinationFrameWindow = parent.frames[e.data.indexIntoParentFrameOfDestination];
        const whatHappened = await sendWasmModule(destinationFrameWindow);
        parent.postMessage(whatHappened, "*");
      } else if (e.data.command === "access document") {
        const destinationFrameWindow = parent.frames[e.data.indexIntoParentFrameOfDestination];
        try {
          destinationFrameWindow.document;
          parent.postMessage("accessed document successfully", "*");
        } catch (e) {
          parent.postMessage(e.name, "*");
        }
      }

      // We could also receive e.data === "WebAssembly.Module message received",
      // but that's handled by await sendWasmModule() above.
    };

    window.onmessageerror = e => {
      e.source.postMessage("messageerror", "*");
    };
    </script>
    """
