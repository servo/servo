registerPaint("argumentsThrows", class {
    static get inputArguments() { throw new TypeError(); }
    paint(ctx, size) { }
});
