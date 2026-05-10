registerPaint("alphaThrows", class {
    static get alpha() { throw new TypeError(); }
    paint(ctx, size) { }
});
