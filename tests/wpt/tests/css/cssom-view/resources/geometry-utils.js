const GEOMETRY_UTILS_EPSILON = 0.5;

function assert_point_approx_equals(actual, expected, description) {
  assert_approx_equals(actual.x, expected.x, GEOMETRY_UTILS_EPSILON,
                       `${description} x`);
  assert_approx_equals(actual.y, expected.y, GEOMETRY_UTILS_EPSILON,
                       `${description} y`);
}

function assert_quad_approx_equals(actual, expected, description) {
  for (const point of ["p1", "p2", "p3", "p4"]) {
    assert_point_approx_equals(actual[point], expected[point],
                               `${description} ${point}`);
  }
}

function assert_rect_approx_equals(actual, expected, description) {
  for (const prop of ["left", "top", "right", "bottom", "width", "height"]) {
    assert_approx_equals(actual[prop], expected[prop], GEOMETRY_UTILS_EPSILON,
                         `${description} ${prop}`);
  }
}

function assert_quad_bounds_match_bounding_rect(element, options, description) {
  const quad = element.getBoxQuads(options)[0];
  assert_rect_approx_equals(quad.getBounds(), element.getBoundingClientRect(),
                            description);
}

function px(value) {
  const result = parseFloat(value);
  return Number.isFinite(result) ? result : 0;
}

function used_box_edges(element) {
  const style = getComputedStyle(element);
  return {
    marginTop: px(style.marginTop),
    marginRight: px(style.marginRight),
    marginBottom: px(style.marginBottom),
    marginLeft: px(style.marginLeft),
    borderTop: px(style.borderTopWidth),
    borderRight: px(style.borderRightWidth),
    borderBottom: px(style.borderBottomWidth),
    borderLeft: px(style.borderLeftWidth),
    paddingTop: px(style.paddingTop),
    paddingRight: px(style.paddingRight),
    paddingBottom: px(style.paddingBottom),
    paddingLeft: px(style.paddingLeft),
    borderWidth: element.offsetWidth,
    borderHeight: element.offsetHeight,
  };
}

function local_box_rect(element, box) {
  const edges = used_box_edges(element);
  const borderWidth = edges.borderWidth;
  const borderHeight = edges.borderHeight;

  if (box === "margin") {
    return {
      x: -edges.marginLeft,
      y: -edges.marginTop,
      width: borderWidth + edges.marginLeft + edges.marginRight,
      height: borderHeight + edges.marginTop + edges.marginBottom,
    };
  }

  if (box === "padding") {
    return {
      x: edges.borderLeft,
      y: edges.borderTop,
      width: borderWidth - edges.borderLeft - edges.borderRight,
      height: borderHeight - edges.borderTop - edges.borderBottom,
    };
  }

  if (box === "content") {
    return {
      x: edges.borderLeft + edges.paddingLeft,
      y: edges.borderTop + edges.paddingTop,
      width: borderWidth - edges.borderLeft - edges.borderRight -
        edges.paddingLeft - edges.paddingRight,
      height: borderHeight - edges.borderTop - edges.borderBottom -
        edges.paddingTop - edges.paddingBottom,
    };
  }

  return {x: 0, y: 0, width: borderWidth, height: borderHeight};
}

function box_size(element, box) {
  const rect = local_box_rect(element, box);
  return {width: rect.width, height: rect.height};
}

function box_origin(element, box) {
  const rect = local_box_rect(element, box);
  return new DOMPoint(rect.x, rect.y);
}

function quad_from_rect(rect) {
  return new DOMQuad(
    new DOMPoint(rect.x, rect.y),
    new DOMPoint(rect.x + rect.width, rect.y),
    new DOMPoint(rect.x + rect.width, rect.y + rect.height),
    new DOMPoint(rect.x, rect.y + rect.height)
  );
}

function translated_quad(quad, dx, dy) {
  return new DOMQuad(
    new DOMPoint(quad.p1.x + dx, quad.p1.y + dy),
    new DOMPoint(quad.p2.x + dx, quad.p2.y + dy),
    new DOMPoint(quad.p3.x + dx, quad.p3.y + dy),
    new DOMPoint(quad.p4.x + dx, quad.p4.y + dy)
  );
}

function translated_point(point, dx, dy) {
  return new DOMPoint(point.x + dx, point.y + dy);
}

function expected_quad_in_target_box(source, target, fromBox, toBox) {
  const targetOrigin = box_origin(target, toBox);
  return translated_quad(
    source.getBoxQuads({box: fromBox, relativeTo: target})[0],
    -targetOrigin.x,
    -targetOrigin.y
  );
}

function expected_point_in_target_box(source, target, fromBox, toBox, pointName) {
  const targetOrigin = box_origin(target, toBox);
  const point = source.getBoxQuads({box: fromBox, relativeTo: target})[0][pointName];
  return translated_point(point, -targetOrigin.x, -targetOrigin.y);
}

function assert_geometry_utils_exist() {
  assert_equals(typeof Element.prototype.getBoxQuads, "function",
                "Element.prototype.getBoxQuads");
  assert_equals(typeof Element.prototype.convertQuadFromNode, "function",
                "Element.prototype.convertQuadFromNode");
  assert_equals(typeof Element.prototype.convertRectFromNode, "function",
                "Element.prototype.convertRectFromNode");
  assert_equals(typeof Element.prototype.convertPointFromNode, "function",
                "Element.prototype.convertPointFromNode");
}
