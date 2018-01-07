registerPaint("testgreen", class {
    paint(ctx, size) {
        try {
            sleep(30); // too long for a paintworklet to init
        } catch (e) {
            console.log("Problem sleeping: " + e);
        }
        // should fail if control reaches here before timeout
        ctx.fillStyle = 'green';
        ctx.fillRect(0, 0, size.width, size.height);
    }
});
