registerPaint("propertiesThrows", class {
    static get inputProperties() { throw new TypeError(); }
    paint(ctx, size) { }
});
