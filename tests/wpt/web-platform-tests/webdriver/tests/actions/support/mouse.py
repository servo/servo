def get_center(rect):
    return {
        "x": rect["width"] / 2 + rect["x"],
        "y": rect["height"] / 2 + rect["y"],
    }
