registerPaint("test", class {
    paint(ctx, size) {
        if ((size.width === 210) && (size.height === 110)) {
            ctx.fillStyle = "green";
            ctx.fillRect(0, 0, size.width, size.height);
        }
    }
});
