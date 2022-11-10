function window_state_context(t) {
    let rect = null;
    let state = "restored";
    t.add_cleanup(async () => {
        if (state === "minimized")
            await restore();
    });
    async function restore() {
        state = "restored";
        await test_driver.set_window_rect(rect);
    }

    async function minimize() {
        state = "minimized";
        rect = await test_driver.minimize_window();
    }

    return {minimize, restore};
}
