#[doc="Applies the appropriate CSS style to boxes."]

import dom::base::{Element, HTMLImageElement, Node};
import dom::rcu::ReaderMethods;
import image::base::load;
import base::{Box, BTree, NTree, LayoutData, BoxTreeReadMethods, SpecifiedStyle};
import style::{default_style_methods, style_methods};

trait ApplyStyleBoxMethods {
    fn apply_style_for_subtree();
    fn apply_style();
}

impl ApplyStyleBoxMethods of ApplyStyleBoxMethods for @Box {
    fn apply_style_for_subtree() {
        self.apply_style();
        for BTree.each_child(self) |child| {
            child.apply_style_for_subtree();
        }
    }

    #[doc="Applies CSS style to a layout box.

      Get the specified style and apply the existing traits to a
      layout box.  If a trait does not exist, calculate the default
      value for the given type of element and use that instead.

     "]
    fn apply_style() {
        // Right now, we only handle images.
        self.node.read(|node| {
            alt node.kind {
              ~Element(element) {
                let style = self.node.get_specified_style();

                self.appearance.background_color = alt style.background_color {
                  some(col) { col }
                  none { node.kind.default_color() }
                };

                alt element.kind {
                  ~HTMLImageElement(*) {
                    alt element.get_attr(~"src") {
                      some(url) {
                        // FIXME: Some sort of BASE HREF support!
                        // FIXME: Parse URLs!
                        // FIXME: Do not load synchronously!
                        #debug("loading image from %s", url);
                        let image = @load(url);
                        self.appearance.background_image = some(image);
                      }
                      none {
                        /* Ignore. */
                      }
                    }
                  }
                  _ { /* Ignore. */ }
                }
              }
              _ { /* Ignore. */ }
            }
        })
    }
}

