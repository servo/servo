registerPaint("testgreen", class {
    paint(ctx, size) {
        ctx.fillStyle = 'green';
        ctx.fillRect(0, 0, size.width, size.height);
        sleep(30); // too long for a paintworklet to init
    }
});
