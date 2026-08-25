registerPaint("test", class {
    paint(ctx, size) {
        ctx.fillStyle = 'green';
        ctx.fillRect(0, 0, size.width, size.height);
    }
});
