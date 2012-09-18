#[doc="Applies the appropriate CSS style to boxes."]

use au = gfx::geometry;
use layout::base::{Box, SpecifiedStyle, BoxTree};
use layout::context::LayoutContext;
use layout::traverse_parallel::top_down_traversal;
use image::ImageHolder;
use resource::image_cache_task::ImageCacheTask;
use std::net::url::Url;

use css::values::*;

trait ResolveMethods<T> {
    pure fn initial() -> T;
}

impl CSSValue<CSSBackgroundColor> : ResolveMethods<CSSBackgroundColor> {
    pure fn initial() -> CSSBackgroundColor { return BgTransparent; }
}

impl CSSValue<CSSDisplay> : ResolveMethods<CSSDisplay> {
    pure fn initial() -> CSSDisplay { return DisplayInline; }
}

impl CSSValue<BoxSizing> : ResolveMethods<BoxSizing> {
    pure fn initial() -> BoxSizing { return BoxAuto; }
}

impl CSSValue<CSSFontSize> : ResolveMethods<CSSFontSize> {
    pure fn initial() -> CSSFontSize { return AbsoluteSize(Medium); }
}


struct StyleApplicator {
    box: @Box,
    reflow: fn~(),
}

// TODO: normalize this into a normal preorder tree traversal function
fn apply_style(layout_ctx: &LayoutContext, box: @Box, reflow: fn~()) {
    let applicator = StyleApplicator {
        box: box,
        reflow: reflow
    };

    applicator.apply_css_style(layout_ctx);
}

// TODO: this is misleadingly-named. It is actually trying to resolve CSS 'inherit' values.

#[doc="A wrapper around a set of functions that can be applied as a top-down traversal of layout
       boxes."]
fn inheritance_wrapper(layout_ctx: &LayoutContext, box : @Box, reflow: fn~()) {
    let applicator = StyleApplicator {
        box: box,
        reflow: reflow
    };
    applicator.apply_style(layout_ctx);
}

/*
fn resolve_fontsize(box : @Box) {
    // TODO: complete this
    return
}

fn resolve_height(box : @Box) -> au {
    let style = box.node.get_style();
    let inherit_val = match box.tree.parent {
        None => au(0),
        Some(parent) => parent.data.computed_size.height
    };

    box.appearance.height = match style.height {
        Initial => style.height.initial(),
        Inherit => inherit_val,
        Specified(val) => match val { // BoxSizing
            BoxPercent(*) | BoxAuto | BoxLength(Px(_)) => val,
            BoxLength(Em(n)) => BoxLength(Px(n * box.appearance.font_size.abs()))
        }
    }
}

fn resolve_width(box : @Box) {
    let style = box.node.get_specified_style();
    let inherit_val = match box.tree.parent {
        None => style.height.initial(),
        Some(node) => node.appearance.width
    };

    box.appearance.width = match style.width {
        Initial => style.width.initial(),
        Inherit => inherit_val,
        Specified(val) => match val { // BoxSizing
            BoxPercent(*) | BoxAuto | BoxLength(Px(_)) => val,
            BoxLength(Em(n)) => BoxLength(Px(n * box.appearance.font_size.abs()))
        }
    }
}*/

impl StyleApplicator {
    fn apply_css_style(layout_ctx: &LayoutContext) {
        let reflow = copy self.reflow;

        do BoxTree.each_child(self.box) |child| {
            inheritance_wrapper(layout_ctx, child, reflow); true
        }
    }

    #[doc="Applies CSS style to a layout box.

      Get the specified style and apply the existing traits to a
      layout box.  If a trait does not exist, calculate the default
      value for the given type of element and use that instead.

     "]
    fn apply_style(layout_ctx: &LayoutContext) {

        // Right now, we only handle images.
        do self.box.node.read |node| {
            match node.kind {
              ~dom::base::Element(element) => {
                match element.kind {
                  ~dom::base::HTMLImageElement(*) => {
                    let url = element.get_attr(~"src");
                    
                    if url.is_some() {
                        // FIXME: Some sort of BASE HREF support!
                        // FIXME: Parse URLs!
                        let new_url = make_url(option::unwrap(url), Some(copy layout_ctx.doc_url));
                        self.box.data.background_image = Some(ImageHolder(new_url, layout_ctx.image_cache, self.reflow))
                    };
                  }
                  _ => { /* Ignore. */ }
                }
              }
              _ => { /* Ignore. */ }
            }
        }
    }
}

#[cfg(test)]
mod test {
    /* TODO: rewrite once cascade and resolve written. */
}