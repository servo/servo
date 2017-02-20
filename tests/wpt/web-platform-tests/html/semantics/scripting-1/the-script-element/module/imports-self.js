import { SelfInner } from "./imports-self-inner.js";

test_importSelf.step(function () {
    assert_unreached("This module should not have loaded!");
});
