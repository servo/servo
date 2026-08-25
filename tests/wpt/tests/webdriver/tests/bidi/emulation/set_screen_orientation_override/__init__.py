def get_angle(_type, natural):
    # Return angle matching screen orientation values lists:
    # https://www.w3.org/TR/screen-orientation/#dfn-screen-orientation-values-lists.
    if natural == "portrait":
        if _type == "portrait-primary":
            return 0
        if _type == "landscape-primary":
            return 90
        if _type == "portrait-secondary":
            return 180
        if _type == "landscape-secondary":
            return 270
    if natural == "landscape":
        if _type == "landscape-primary":
            return 0
        if _type == "portrait-primary":
            return 90
        if _type == "landscape-secondary":
            return 180
        if _type == "portrait-secondary":
            return 270
    raise Exception(
        f"Unexpected screen orientation type: {_type} with natural orientation: {natural}")
