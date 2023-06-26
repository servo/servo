onmessage = async (event) => {
  const readableStream = event.data.readableStream;
  const writableStream = event.data.writableStream;
  const insertError = event.data.insertError;

  try {
    await readableStream.pipeThrough(new TransformStream({
      transform: (chunk, controller) => {
        if (insertError) {
          controller.enqueue("This is not a valid frame");
        } else {
          controller.enqueue(chunk);
        }
      }
    })).pipeTo(writableStream);

    postMessage({success:true});
  } catch(e) {
    postMessage({success:false, error: e});
  }

}
