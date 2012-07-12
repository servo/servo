#[doc="Applies the appropriate CSS style to boxes."]

import dom::base::{Element, HTMLImageElement, Node};
import dom::rcu::ReaderMethods;
import image::base::load;
import base::{Box, BTree, NTree, LayoutData, BoxTreeReadMethods};
import style::style_methods;

impl ApplyStyleBoxMethods for @Box {
    fn apply_style_for_subtree() {
        self.apply_style();
        for BTree.each_child(self) |child| {
            child.apply_style_for_subtree();
        }
    }

    #[doc="Applies CSS style."]
    fn apply_style() {
        // Right now, we only handle images.
        self.node.read(|node| {
            alt node.kind {
              ~Element(element) {
                let style = self.node.get_computed_style();

                self.appearance.background_color = some(style.back_color);

                alt element.kind {
                  ~HTMLImageElement(*) {
                    alt element.get_attr("src") {
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

