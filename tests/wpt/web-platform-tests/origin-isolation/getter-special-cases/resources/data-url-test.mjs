import { insertCustomIframe, testSupportScript } from "./helpers.mjs";
import { testOriginIsolationRestricted } from "../../resources/helpers.mjs";

export default () => {
  promise_setup(() => {
    return insertCustomIframe(`data:text/html,${testSupportScript}`);
  });

  // The data: URL iframe has an opaque origin, so it should return true, since
  // for them site === origin so they are always "origin-isolated".

  testOriginIsolationRestricted(0, true, "data: URL child");
};
