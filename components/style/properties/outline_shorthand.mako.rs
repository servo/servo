<%page args="helpers" />

<%helpers:shorthand name="outline" sub_properties="outline-color outline-style outline-width">
    use properties::longhands::outline_width;
    use values::specified;

    let _unused = context;
    let mut color = None;
    let mut style = None;
    let mut width = None;
    let mut any = false;
    loop {
        if color.is_none() {
            if let Ok(value) = input.try(specified::CSSColor::parse) {
                color = Some(value);
                any = true;
                continue
            }
        }
        if style.is_none() {
            if let Ok(value) = input.try(specified::BorderStyle::parse) {
                style = Some(value);
                any = true;
                continue
            }
        }
        if width.is_none() {
            if let Ok(value) = input.try(|input| outline_width::parse(context, input)) {
                width = Some(value);
                any = true;
                continue
            }
        }
        break
    }
    if any {
        Ok(Longhands {
            outline_color: color,
            outline_style: style,
            outline_width: width,
        })
    } else {
        Err(())
    }
</%helpers:shorthand>
