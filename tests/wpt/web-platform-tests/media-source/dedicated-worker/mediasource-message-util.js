// This script provides an object with common message subjects to assist main
// and worker thread communication.

const messageSubject = {
  ERROR: "error",  // info field may contain more detail
  OBJECT_URL: "object url", // info field contains object URL
  STARTED_BUFFERING: "started buffering",
  FINISHED_BUFFERING: "finished buffering",
  VERIFY_DURATION: "verify duration", // info field contains expected duration
  AWAIT_DURATION: "await duration", // wait for element duration to match the expected duration in the info field
  VERIFY_HAVE_NOTHING: "verify have nothing readyState",
  VERIFY_AT_LEAST_HAVE_METADATA: "verify readyState is at least HAVE_METADATA",
  ACK_VERIFIED: "verified", // info field contains the message values that requested the verification
  WORKER_DONE: "worker done", // this lets worker signal main to successfully end the test
};
