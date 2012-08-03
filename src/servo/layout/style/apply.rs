#[doc="Applies the appropriate CSS style to boxes."]

import dom::base::{Element, HTMLImageElement, Node};
import dom::rcu::ReaderMethods;
import either::right;
import image::base::load;
import base::{Box, BTree, NTree, LayoutData, BoxTreeReadMethods, SpecifiedStyle, ImageHolder};
import style::{default_style_methods, style_methods};
import traverse::top_down_traversal;

trait ApplyStyleBoxMethods {
    fn apply_css_style();
    fn apply_style();
}

#[doc="A wrapper so the function can be passed around by name."]
fn apply_style_wrapper(box : @Box) {
    box.apply_style();
}

impl ApplyStyleBoxMethods of ApplyStyleBoxMethods for @Box {
    fn apply_css_style() {
        top_down_traversal(self, apply_style_wrapper);
    }

    #[doc="Applies CSS style to a layout box.

      Get the specified style and apply the existing traits to a
      layout box.  If a trait does not exist, calculate the default
      value for the given type of element and use that instead.

     "]
    fn apply_style() {
        // Right now, we only handle images.
        do self.node.read |node| {
            alt node.kind {
              ~Element(element) => {
                let style = self.node.get_specified_style();

                self.appearance.background_color = alt style.background_color {
                  some(col) => col,
                  none => node.kind.default_color()
                };

                alt element.kind {
                  ~HTMLImageElement(*) => {
                    let url = element.get_attr(~"src");
                    
                    if url.is_some() {
                        // FIXME: Some sort of BASE HREF support!
                        // FIXME: Parse URLs!
                        self.appearance.background_image = some(ImageHolder(option::unwrap(url)))
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
