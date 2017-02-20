import { CycleA } from "./imports-cycle-a.js";

test_importCycle.step(function () {
    assert_unreached("This module should not have loaded!");
});
