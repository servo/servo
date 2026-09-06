// The operation is chosen by the `op` search parameter, and only that operation
// runs: a subtest must not depend on an earlier operation settling, and store()
// has a side effect that should not fire for the get() and create() subtests.
(async () => {
  const params = new URL(location.href).searchParams;
  const id = params.get("id");
  const op = params.get("op");
  const report = {
    id,
    op,
    origin: String(self.origin),
    secure: self.isSecureContext,
    supported: false,
    result: "unsupported",
  };

  const post = () => (window.top || window.parent).postMessage(report, "*");

  try {
    if (!self.PasswordCredential || !navigator.credentials) {
      post();
      return;
    }
    report.supported = true;

    const data = { id: "id", password: "pencil" };

    // Built before the operation runs so a constructor failure is never
    // reported as a store() result.
    let operation;
    if (op === "get") {
      operation = () => navigator.credentials.get({ password: true });
    } else if (op === "create") {
      operation = () => navigator.credentials.create({ password: data });
    } else if (op === "store") {
      let credential;
      try {
        credential = new PasswordCredential(data);
      } catch (error) {
        report.result = "construction-failed:" + error.name;
        post();
        return;
      }
      operation = () => navigator.credentials.store(credential);
    } else {
      report.result = "unknown-op";
      post();
      return;
    }

    try {
      await operation();
      report.result = "resolved";
    } catch (error) {
      report.result = error.name;
    }
  } catch (error) {
    report.result = "harness-error:" + error.name;
  }

  post();
})();
