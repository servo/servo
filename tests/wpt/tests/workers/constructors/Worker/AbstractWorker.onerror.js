for (;) // should cause onerror to be invoked, but onerror is null, so
        // the error is "not handled". should fire an ErrorEvent on the
        // worker.
  break;
postMessage(1); // shouldn't do anything since the script doesn't compile