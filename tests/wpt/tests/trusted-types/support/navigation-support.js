function navigateToJavascriptURL(reportOnly) {
    const params = new URLSearchParams(location.search);

    if (!!params.get("defaultpolicy")) {
        trustedTypes.createPolicy("default", {
            createScript: s => {
                return s.replace("continue", "defaultpolicywashere")
            },
        });
    }

    function bounceEventToOpener(e) {
        const msg = {};
        for (const field of ["effectiveDirective", "sample", "type"]) {
            msg[field] = e[field];
        }

        msg["uri"] = location.href;
        window.opener.postMessage(msg, "*");
    }

    // If a navigation is blocked by Trusted Types, we expect this window to
    // throw a SecurityPolicyViolationEvent. If it's not blocked, we expect the
    // loaded frame to through DOMContentLoaded. In either case there should be
    // _some_ event that we can expect.
    document.addEventListener("DOMContentLoaded", bounceEventToOpener);
    // Prevent loops.
    if (params.has("navigationattempted")) {
      return;
    }

    let url = new URL(
      reportOnly ?
      // Navigate to the non-report-only version of the test. That has the same
      // event listening setup as this, but is a different target URI.
      location.href.replace("-report-only", "") :
      // We'll use a javascript:-url to navigate to ourselves, so that we can
      // re-use the messageing mechanisms above.
      location.href
    );
    url.searchParams.set("navigationattempted", 1);
    url.searchParams.set("continue", 1);
    let target_script = `location.href='${url.toString()}';`;

    function getAndPreparareNavigationElement(javaScriptURL) {
        let target = "_self";
        if (!!params.get("frame")) {
            const frame = document.createElement("iframe");
            frame.src = "frame-without-trusted-types.html";
            frames.name = "frame";
            document.body.appendChild(frame);
            target = "frame";
        }

        if (!!params.get("form-submission")) {
            const submit = document.getElementById("submit");

            // Careful, the IDL attributes are defined in camel-case.
            submit.formAction = javaScriptURL;
            submit.formTarget = target;

            return submit;
        }

        const anchor = document.getElementById("anchor");
        anchor.href = javaScriptURL;
        anchor.target = target;
        return anchor;
    }

    const navigationElement = getAndPreparareNavigationElement(`javascript:${target_script}`);
    document.addEventListener("DOMContentLoaded", async _ => {
      let {violations, exception} =
        await trusted_type_violations_and_exception_for(_ => navigationElement.click());
      violations.forEach(violationEvent => bounceEventToOpener(violationEvent));
      if (!params.get("defaultpolicy") && violations.length == 0) {
        window.opener.postMessage("No securitypolicyviolation reported!", "*");
      }
    });
}
