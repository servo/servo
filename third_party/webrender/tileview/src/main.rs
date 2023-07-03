/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// Command line tool to convert logged tile cache files into a visualization.
///
/// Steps to use this:
/// 1. enable webrender; enable gfx.webrender.debug.tile-cache-logging
/// 2. take a capture using ctrl-shift-3
///    if all is well, there will be a .../wr-capture/tilecache folder with *.ron files
/// 3. run tileview with that folder as the first parameter and some empty output folder as the
///    2nd:
///    cargo run --release -- /foo/bar/wr-capture/tilecache /tmp/tilecache
/// 4. open /tmp/tilecache/index.html
///
/// Note: accurate interning info requires that the circular buffer doesn't wrap around.
/// So for best results, use this workflow:
/// a. start up blank browser; in about:config enable logging; close browser
/// b. start new browser, quickly load the repro
/// c. capture.
///
/// If that's tricky, you can also just throw more memory at it: in render_backend.rs,
/// increase the buffer size here: 'TileCacheLogger::new(500usize)'
///
/// Note: some features don't work when opening index.html directly due to cross-scripting
/// protections.  Instead use a HTTP server:
///     python -m SimpleHTTPServer 8000


use webrender::{TileNode, TileNodeKind, InvalidationReason, TileOffset};
use webrender::{TileSerializer, TileCacheInstanceSerializer, TileCacheLoggerUpdateLists};
use webrender::{PrimitiveCompareResultDetail, CompareHelperResult, ItemUid};
use serde::Deserialize;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::ffi::OsString;
use std::collections::HashMap;
use webrender::api::{enumerate_interners, ColorF};
use euclid::{Rect, Transform3D};
use webrender_api::units::{PicturePoint, PictureSize, PicturePixel, WorldPixel};

static RES_JAVASCRIPT: &'static str = include_str!("tilecache.js");
static RES_BASE_CSS: &'static str   = include_str!("tilecache_base.css");

#[derive(Deserialize)]
pub struct Slice {
    pub transform: Transform3D<f32, PicturePixel, WorldPixel>,
    pub tile_cache: TileCacheInstanceSerializer
}

// invalidation reason CSS colors
static CSS_FRACTIONAL_OFFSET: &str       = "fill:#4040c0;fill-opacity:0.1;";
static CSS_BACKGROUND_COLOR: &str        = "fill:#10c070;fill-opacity:0.1;";
static CSS_SURFACE_OPACITY_CHANNEL: &str = "fill:#c040c0;fill-opacity:0.1;";
static CSS_NO_TEXTURE: &str              = "fill:#c04040;fill-opacity:0.1;";
static CSS_NO_SURFACE: &str              = "fill:#40c040;fill-opacity:0.1;";
static CSS_PRIM_COUNT: &str              = "fill:#40f0f0;fill-opacity:0.1;";
static CSS_CONTENT: &str                 = "fill:#f04040;fill-opacity:0.1;";
static CSS_COMPOSITOR_KIND_CHANGED: &str = "fill:#f0c070;fill-opacity:0.1;";
static CSS_VALID_RECT_CHANGED: &str      = "fill:#ff00ff;fill-opacity:0.1;";

// parameters to tweak the SVG generation
struct SvgSettings {
    pub scale: f32,
    pub x: f32,
    pub y: f32,
}

fn tile_node_to_svg(node: &TileNode,
                    transform: &Transform3D<f32, PicturePixel, WorldPixel>,
                    svg_settings: &SvgSettings) -> String
{
    match &node.kind {
        TileNodeKind::Leaf { .. } => {
            let rect_world = transform.outer_transformed_rect(&node.rect.to_rect()).unwrap();
            format!("<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" />\n",
                    rect_world.origin.x    * svg_settings.scale + svg_settings.x,
                    rect_world.origin.y    * svg_settings.scale + svg_settings.y,
                    rect_world.size.width  * svg_settings.scale,
                    rect_world.size.height * svg_settings.scale)
        },
        TileNodeKind::Node { children } => {
            children.iter().fold(String::new(), |acc, child| acc + &tile_node_to_svg(child, transform, svg_settings) )
        }
    }
}

fn tile_to_svg(key: TileOffset,
               tile: &TileSerializer,
               slice: &Slice,
               prev_tile: Option<&TileSerializer>,
               itemuid_to_string: &HashMap<ItemUid, String>,
               tile_stroke: &str,
               prim_class: &str,
               invalidation_report: &mut String,
               svg_width: &mut i32, svg_height: &mut i32,
               svg_settings: &SvgSettings) -> String
{
    let mut svg = format!("\n<!-- tile key {},{} ; -->\n", key.x, key.y);


    let tile_fill =
        match tile.invalidation_reason {
            Some(InvalidationReason::FractionalOffset { .. }) => CSS_FRACTIONAL_OFFSET.to_string(),
            Some(InvalidationReason::BackgroundColor { .. }) => CSS_BACKGROUND_COLOR.to_string(),
            Some(InvalidationReason::SurfaceOpacityChanged { .. }) => CSS_SURFACE_OPACITY_CHANNEL.to_string(),
            Some(InvalidationReason::NoTexture) => CSS_NO_TEXTURE.to_string(),
            Some(InvalidationReason::NoSurface) => CSS_NO_SURFACE.to_string(),
            Some(InvalidationReason::PrimCount { .. }) => CSS_PRIM_COUNT.to_string(),
            Some(InvalidationReason::CompositorKindChanged) => CSS_COMPOSITOR_KIND_CHANGED.to_string(),
            Some(InvalidationReason::Content { .. } ) => CSS_CONTENT.to_string(),
            Some(InvalidationReason::ValidRectChanged) => CSS_VALID_RECT_CHANGED.to_string(),
            None => {
                let mut background = tile.background_color;
                if background.is_none() {
                    background = slice.tile_cache.background_color
                }
                match background {
                   Some(color) => {
                       let rgb = ( (color.r * 255.0) as u8,
                                   (color.g * 255.0) as u8,
                                   (color.b * 255.0) as u8 );
                       format!("fill:rgb({},{},{});fill-opacity:0.3;", rgb.0, rgb.1, rgb.2)
                   }
                   None => "fill:none;".to_string()
                }
            }
        };

    //let tile_style = format!("{}{}", tile_fill, tile_stroke);
    let tile_style = format!("{}stroke:none;", tile_fill);

    let title = match tile.invalidation_reason {
        Some(_) => format!("<title>slice {} tile ({},{}) - {:?}</title>",
                            slice.tile_cache.slice, key.x, key.y,
                            tile.invalidation_reason),
        None => String::new()
    };

    if let Some(reason) = &tile.invalidation_reason {
        invalidation_report.push_str(
            &format!("<div class=\"subheader\">slice {} key ({},{})</div><div class=\"data\">",
                     slice.tile_cache.slice,
                     key.x, key.y));

        // go through most reasons individually so we can print something nicer than
        // the default debug formatting of old and new:
        match reason {
            InvalidationReason::FractionalOffset { old, new } => {
                invalidation_report.push_str(
                    &format!("<b>FractionalOffset</b> changed from ({},{}) to ({},{})",
                             old.x, old.y, new.x, new.y));
            },
            InvalidationReason::BackgroundColor { old, new } => {
                fn to_str(c: &Option<ColorF>) -> String {
                    if let Some(c) = c {
                        format!("({},{},{},{})", c.r, c.g, c.b, c.a)
                    } else {
                        "none".to_string()
                    }
                }

                invalidation_report.push_str(
                    &format!("<b>BackGroundColor</b> changed from {} to {}",
                             to_str(old), to_str(new)));
            },
            InvalidationReason::SurfaceOpacityChanged { became_opaque } => {
                invalidation_report.push_str(
                    &format!("<b>SurfaceOpacityChanged</b> changed from {} to {}",
                             !became_opaque, became_opaque));
            },
            InvalidationReason::PrimCount { old, new } => {
                // diff the lists to find removed and added ItemUids,
                // and convert them to strings to pretty-print what changed:
                let old = old.as_ref().unwrap();
                let new = new.as_ref().unwrap();
                let removed = old.iter()
                                 .filter(|i| !new.contains(i))
                                 .fold(String::new(),
                                       |acc, i| acc + "<li>" + &(i.get_uid()).to_string() + "..."
                                                    + &itemuid_to_string.get(i).unwrap_or(&String::new())
                                                    + "</li>\n");
                let added   = new.iter()
                                 .filter(|i| !old.contains(i))
                                 .fold(String::new(),
                                       |acc, i| acc + "<li>" + &(i.get_uid()).to_string() + "..."
                                                    + &itemuid_to_string.get(i).unwrap_or(&String::new())
                                                    + "</li>\n");
                invalidation_report.push_str(
                    &format!("<b>PrimCount</b> changed from {} to {}:<br/>\
                              removed:<ul>{}</ul>
                              added:<ul>{}</ul>",
                              old.len(), new.len(),
                              removed, added));
            },
            InvalidationReason::Content { prim_compare_result, prim_compare_result_detail } => {
                let _ = prim_compare_result;
                match prim_compare_result_detail {
                    Some(PrimitiveCompareResultDetail::Descriptor { old, new }) => {
                        if old.prim_uid == new.prim_uid {
                            // if the prim uid hasn't changed then try to print something useful
                            invalidation_report.push_str(
                                &format!("<b>Content: Descriptor</b> changed for uid {}<br/>",
                                         old.prim_uid.get_uid()));
                            let mut changes = String::new();
                            if old.prim_clip_box != new.prim_clip_box {
                                changes += &format!("<li><b>prim_clip_rect</b> changed from {},{} -> {},{}",
                                                    old.prim_clip_box.min.x,
                                                    old.prim_clip_box.min.y,
                                                    old.prim_clip_box.max.x,
                                                    old.prim_clip_box.max.y);
                                changes += &format!(" to {},{} -> {},{}</li>",
                                                    new.prim_clip_box.min.x,
                                                    new.prim_clip_box.min.y,
                                                    new.prim_clip_box.max.x,
                                                    new.prim_clip_box.max.y);
                            }
                            invalidation_report.push_str(
                                &format!("<ul>{}<li>Item: {}</li></ul>",
                                             changes,
                                             &itemuid_to_string.get(&old.prim_uid).unwrap_or(&String::new())));
                        } else {
                            // .. if prim UIDs have changed, just dump both items and descriptors.
                            invalidation_report.push_str(
                                &format!("<b>Content: Descriptor</b> changed; old uid {}, new uid {}:<br/>",
                                             old.prim_uid.get_uid(),
                                             new.prim_uid.get_uid()));
                            invalidation_report.push_str(
                                &format!("old:<ul><li>Desc: {:?}</li><li>Item: {}</li></ul>",
                                             old,
                                             &itemuid_to_string.get(&old.prim_uid).unwrap_or(&String::new())));
                            invalidation_report.push_str(
                                &format!("new:<ul><li>Desc: {:?}</li><li>Item: {}</li></ul>",
                                             new,
                                             &itemuid_to_string.get(&new.prim_uid).unwrap_or(&String::new())));
                        }
                    },
                    Some(PrimitiveCompareResultDetail::Clip { detail }) => {
                        match detail {
                            CompareHelperResult::Count { prev_count, curr_count } => {
                                invalidation_report.push_str(
                                    &format!("<b>Content: Clip</b> count changed from {} to {}<br/>",
                                             prev_count, curr_count ));
                            },
                            CompareHelperResult::NotEqual { prev, curr } => {
                                invalidation_report.push_str(
                                    &format!("<b>Content: Clip</b> ItemUids changed from {} to {}:<br/>",
                                             prev.get_uid(), curr.get_uid() ));
                                invalidation_report.push_str(
                                    &format!("old:<ul><li>{}</li></ul>",
                                             &itemuid_to_string.get(&prev).unwrap_or(&String::new())));
                                invalidation_report.push_str(
                                    &format!("new:<ul><li>{}</li></ul>",
                                             &itemuid_to_string.get(&curr).unwrap_or(&String::new())));
                            },
                            reason => {
                                invalidation_report.push_str(&format!("{:?}", reason));
                            },
                        }
                    },
                    reason => {
                        invalidation_report.push_str(&format!("{:?}", reason));
                    },
                }
            },
            reason => {
                invalidation_report.push_str(&format!("{:?}", reason));
            },
        }
        invalidation_report.push_str("</div>\n");
    }

    svg += &format!(r#"<rect x="{}" y="{}" width="{}" height="{}" style="{}" ></rect>"#,
            tile.rect.origin.x    * svg_settings.scale + svg_settings.x,
            tile.rect.origin.y    * svg_settings.scale + svg_settings.y,
            tile.rect.size.width  * svg_settings.scale,
            tile.rect.size.height * svg_settings.scale,
            tile_style);

    svg += &format!("\n\n<g class=\"svg_quadtree\">\n{}</g>\n",
                   tile_node_to_svg(&tile.root, &slice.transform, svg_settings));

    let right  = (tile.rect.origin.x + tile.rect.size.width) as i32;
    let bottom = (tile.rect.origin.y + tile.rect.size.height) as i32;

    *svg_width  = if right  > *svg_width  { right  } else { *svg_width  };
    *svg_height = if bottom > *svg_height { bottom } else { *svg_height };

    svg += "\n<!-- primitives -->\n";

    svg += &format!("<g id=\"{}\">\n\t", prim_class);


    let rect_visual_id = Rect {
        origin: tile.rect.origin,
        size: PictureSize::new(1.0, 1.0)
    };
    let rect_visual_id_world = slice.transform.outer_transformed_rect(&rect_visual_id).unwrap();
    svg += &format!("\n<text class=\"svg_tile_visual_id\" x=\"{}\" y=\"{}\">{},{} ({})</text>",
            rect_visual_id_world.origin.x           * svg_settings.scale + svg_settings.x,
            (rect_visual_id_world.origin.y + 110.0) * svg_settings.scale + svg_settings.y,
            key.x, key.y, slice.tile_cache.slice);


    for prim in &tile.current_descriptor.prims {
        let rect = prim.prim_clip_box;

        // the transform could also be part of the CSS, let the browser do it;
        // might be a bit faster and also enable actual 3D transforms.
        let rect_pixel = Rect {
            origin: PicturePoint::new(rect.min.x, rect.min.y),
            size: PictureSize::new(rect.max.x - rect.min.x, rect.max.y - rect.min.y),
        };
        let rect_world = slice.transform.outer_transformed_rect(&rect_pixel).unwrap();

        let style =
            if let Some(prev_tile) = prev_tile {
                // when this O(n^2) gets too slow, stop brute-forcing and use a set or something
                if prev_tile.current_descriptor.prims.iter().find(|&prim| prim.prim_clip_box == rect).is_some() {
                    ""
                } else {
                    "class=\"svg_changed_prim\" "
                }
            } else {
                "class=\"svg_changed_prim\" "
            };

        svg += &format!("<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" {}/>",
                        rect_world.origin.x    * svg_settings.scale + svg_settings.x,
                        rect_world.origin.y    * svg_settings.scale + svg_settings.y,
                        rect_world.size.width  * svg_settings.scale,
                        rect_world.size.height * svg_settings.scale,
                        style);

        svg += "\n\t";
    }

    svg += "\n</g>\n";

    // nearly invisible, all we want is the toolip really
    let style = "style=\"fill-opacity:0.001;";
    svg += &format!("<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" {}{}\" >{}<\u{2f}rect>",
                    tile.rect.origin.x    * svg_settings.scale + svg_settings.x,
                    tile.rect.origin.y    * svg_settings.scale + svg_settings.y,
                    tile.rect.size.width  * svg_settings.scale,
                    tile.rect.size.height * svg_settings.scale,
                    style,
                    tile_stroke,
                    title);

    svg
}

fn slices_to_svg(slices: &[Slice], prev_slices: Option<Vec<Slice>>,
                 itemuid_to_string: &HashMap<ItemUid, String>,
                 svg_width: &mut i32, svg_height: &mut i32,
                 max_slice_index: &mut usize,
                 svg_settings: &SvgSettings) -> (String, String)
{
    let svg_begin = "<?xml\u{2d}stylesheet type\u{3d}\"text/css\" href\u{3d}\"tilecache_base.css\" ?>\n\
                     <?xml\u{2d}stylesheet type\u{3d}\"text/css\" href\u{3d}\"tilecache.css\" ?>\n";

    let mut svg = String::new();
    let mut invalidation_report = "<div class=\"header\">Invalidation</div>\n".to_string();

    for slice in slices {
        let tile_cache = &slice.tile_cache;
        *max_slice_index = if tile_cache.slice > *max_slice_index { tile_cache.slice } else { *max_slice_index };

        invalidation_report.push_str(&format!("<div id=\"invalidation_slice{}\">\n", tile_cache.slice));

        let prim_class = format!("tile_slice{}", tile_cache.slice);

        svg += &format!("\n<g id=\"tile_slice{}_everything\">", tile_cache.slice);

        //println!("slice {}", tile_cache.slice);
        svg += &format!("\n<!-- tile_cache slice {} -->\n",
                              tile_cache.slice);

        //let tile_stroke = "stroke:grey;stroke-width:1;".to_string();
        let tile_stroke = "stroke:none;".to_string();

        let mut prev_slice = None;
        if let Some(prev) = &prev_slices {
            for prev_search in prev {
                if prev_search.tile_cache.slice == tile_cache.slice {
                    prev_slice = Some(prev_search);
                    break;
                }
            }
        }

        for (key, tile) in &tile_cache.tiles {
            let mut prev_tile = None;
            if let Some(prev) = prev_slice {
                prev_tile = prev.tile_cache.tiles.get(key);
            }

            svg += &tile_to_svg(*key, &tile, &slice, prev_tile,
                                      itemuid_to_string,
                                      &tile_stroke, &prim_class,
                                      &mut invalidation_report,
                                      svg_width, svg_height, svg_settings);
        }

        svg += "\n</g>";

        invalidation_report.push_str("</div>\n");
    }

    (
        format!("{}<svg version=\"1.1\" baseProfile=\"full\" xmlns=\"http://www.w3.org/2000/svg\" \
                width=\"{}\" height=\"{}\" >",
                    svg_begin,
                    svg_width,
                    svg_height)
            + "\n"
            + "<rect fill=\"black\" width=\"100%\" height=\"100%\"/>\n"
            + &svg
            + "\n</svg>\n",
        invalidation_report
    )
}

fn write_html(output_dir: &Path, max_slice_index: usize, svg_files: &[String], intern_files: &[String]) {
    let html_head = "<!DOCTYPE html>\n\
                     <html>\n\
                     <head>\n\
                     <meta charset=\"UTF-8\">\n\
                     <link rel=\"stylesheet\" type=\"text/css\" href=\"tilecache_base.css\"></link>\n\
                     <link rel=\"stylesheet\" type=\"text/css\" href=\"tilecache.css\"></link>\n\
                     </head>\n"
                     .to_string();

    let html_body = "<body bgcolor=\"#000000\" onload=\"load()\">\n"
                     .to_string();


    let mut script = "\n<script>\n".to_string();

    script = format!("{}var svg_files = [\n", script);
    for svg_file in svg_files {
        script = format!("{}    \"{}\",\n", script, svg_file);
    }
    script = format!("{}];\n\n", script);

    script = format!("{}var intern_files = [\n", script);
    for intern_file in intern_files {
        script = format!("{}    \"{}\",\n", script, intern_file);
    }
    script = format!("{}];\n</script>\n\n", script);

    script = format!("{}<script src=\"tilecache.js\" type=\"text/javascript\"></script>\n\n", script);


    let html_end   = "</body>\n\
                      </html>\n"
                      .to_string();

    let mut html_slices_form =
            "\n<form id=\"slicecontrols\">\n\
                Slice\n".to_string();

    for ix in 0..max_slice_index + 1 {
        html_slices_form +=
            &format!(
                "<input id=\"slice_toggle{}\" \
                        type=\"checkbox\" \
                        onchange=\"update_slice_visibility({})\" \
                        checked=\"checked\" />\n\
                <label for=\"slice_toggle{}\">{}</label>\n",
                ix,
                max_slice_index + 1,
                ix,
                ix );
    }

    html_slices_form += "<form>\n";

    let html_body = format!(
        "{}\n\
        <div class=\"split left\">\n\
            <div>\n\
                <object id=\"svg_container0\" type=\"image/svg+xml\" data=\"{}\" class=\"tile_svg\" ></object>\n\
                <object id=\"svg_container1\" type=\"image/svg+xml\" data=\"{}\" class=\"tile_svg\" ></object>\n\
            </div>\n\
        </div>\n\
        \n\
        <div class=\"split right\">\n\
            <iframe width=\"100%\" id=\"intern\" src=\"{}\"></iframe>\n\
        </div>\n\
        \n\
        <div id=\"svg_ui_overlay\">\n\
            <div id=\"text_frame_counter\">{}</div>\n\
            <div id=\"text_spacebar\">Spacebar to Play</div>\n\
            <div>Use Left/Right to Step</div>\n\
            <input id=\"frame_slider\" type=\"range\" min=\"0\" max=\"{}\" value=\"0\" class=\"svg_ui_slider\" />
            {}
        </div>",
        html_body,
        svg_files[0],
        svg_files[0],
        intern_files[0],
        svg_files[0],
        svg_files.len(),
        html_slices_form );

    let html = format!("{}{}{}{}", html_head, html_body, script, html_end);

    let output_file = output_dir.join("index.html");
    let mut html_output = File::create(output_file).unwrap();
    html_output.write_all(html.as_bytes()).unwrap();
}

fn write_css(output_dir: &Path, max_slice_index: usize, svg_settings: &SvgSettings) {
    let mut css = String::new();

    for ix in 0..max_slice_index + 1 {
        let color = ( ix % 7 ) + 1;
        let rgb = format!("rgb({},{},{})",
                            if color & 2 != 0 { 205 } else { 90 },
                            if color & 4 != 0 { 205 } else { 90 },
                            if color & 1 != 0 { 225 } else { 90 });

        let prim_class = format!("tile_slice{}", ix);

        css += &format!("#{} {{\n\
                           fill: {};\n\
                           fill-opacity: 0.03;\n\
                           stroke-width: {};\n\
                           stroke: {};\n\
                        }}\n\n",
                        prim_class,
                        //rgb,
                        "none",
                        0.8 * svg_settings.scale,
                        rgb);
    }

    css += &format!(".svg_tile_visual_id {{\n\
                         font: {}px sans-serif;\n\
                         fill: rgb(50,50,50);\n\
                     }}\n\n",
                     150.0 * svg_settings.scale);

    let output_file = output_dir.join("tilecache.css");
    let mut css_output = File::create(output_file).unwrap();
    css_output.write_all(css.as_bytes()).unwrap();
}

macro_rules! updatelist_to_html_macro {
    ( $( $name:ident: $ty:ty, )+ ) => {
        fn updatelist_to_html(update_lists: &TileCacheLoggerUpdateLists,
                              invalidation_report: String) -> String
        {
            let mut html = "\
                <!DOCTYPE html>\n\
                <html> <head> <meta charset=\"UTF-8\">\n\
                <link rel=\"stylesheet\" type=\"text/css\" href=\"tilecache_base.css\"></link>\n\
                <link rel=\"stylesheet\" type=\"text/css\" href=\"tilecache.css\"></link>\n\
                </head> <body>\n\
                <div class=\"datasheet\">\n".to_string();

            html += &invalidation_report;

            html += "<div class=\"header\">Interning</div>\n";
            $(
                html += &format!("<div class=\"subheader\">{}</div>\n<div class=\"intern data\">\n",
                                 stringify!($name));
                for list in &update_lists.$name.1 {
                    for insertion in &list.insertions {
                        html += &format!("<div class=\"insert\"><b>{}</b> {}</div>\n",
                                         insertion.uid.get_uid(),
                                         format!("({:?})", insertion.value));
                    }

                    for removal in &list.removals {
                        html += &format!("<div class=\"remove\"><b>{}</b></div>\n",
                                         removal.uid.get_uid());
                    }
                }
                html += "</div><br/>\n";
            )+
            html += "</div> </body> </html>\n";
            html
        }
    }
}
enumerate_interners!(updatelist_to_html_macro);

fn write_tile_cache_visualizer_svg(entry: &std::fs::DirEntry, output_dir: &Path,
                                   slices: &[Slice], prev_slices: Option<Vec<Slice>>,
                                   itemuid_to_string: &HashMap<ItemUid, String>,
                                   svg_width: &mut i32, svg_height: &mut i32,
                                   max_slice_index: &mut usize,
                                   svg_files: &mut Vec::<String>,
                                   svg_settings: &SvgSettings) -> String
{
    let (svg, invalidation_report) = slices_to_svg(&slices, prev_slices,
                                                   itemuid_to_string,
                                                   svg_width, svg_height,
                                                   max_slice_index,
                                                   svg_settings);

    let mut output_filename = OsString::from(entry.path().file_name().unwrap());
    output_filename.push(".svg");
    svg_files.push(output_filename.to_string_lossy().to_string());

    output_filename = output_dir.join(output_filename).into_os_string();
    let mut svg_output = File::create(output_filename).unwrap();
    svg_output.write_all(svg.as_bytes()).unwrap();

    invalidation_report
}

fn write_update_list_html(entry: &std::fs::DirEntry, output_dir: &Path,
                          update_lists: &TileCacheLoggerUpdateLists,
                          html_files: &mut Vec::<String>,
                          invalidation_report: String)
{
    let html = updatelist_to_html(update_lists, invalidation_report);

    let mut output_filename = OsString::from(entry.path().file_name().unwrap());
    output_filename.push(".html");
    html_files.push(output_filename.to_string_lossy().to_string());

    output_filename = output_dir.join(output_filename).into_os_string();
    let mut html_output = File::create(output_filename).unwrap();
    html_output.write_all(html.as_bytes()).unwrap();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        println!("Usage: tileview input_dir output_dir [scale [x y]]");
        println!("    where input_dir is a tile_cache folder inside a wr-capture.");
        println!("    Scale is an optional scaling factor to compensate for high-DPI.");
        println!("    X, Y is an optional offset to shift the entire SVG by.");
        println!("\nexample: cargo run c:/Users/me/AppData/Local/wr-capture.6/tile_cache/ c:/temp/tilecache/");
        std::process::exit(1);
    }

    let input_dir = Path::new(&args[1]);
    let output_dir = Path::new(&args[2]);
    std::fs::create_dir_all(output_dir).unwrap();

    let scale = if args.len() >= 4 { args[3].parse::<f32>().unwrap() } else { 1.0 };
    let x     = if args.len() >= 6 { args[4].parse::<f32>().unwrap() } else { 0.0 }; // >= 6, requires X and Y
    let y     = if args.len() >= 6 { args[5].parse::<f32>().unwrap() } else { 0.0 };
    let svg_settings = SvgSettings { scale, x, y };

    let mut svg_width = 100i32;
    let mut svg_height = 100i32;
    let mut max_slice_index = 0;

    let mut entries: Vec<_> = std::fs::read_dir(input_dir).unwrap()
                                                          .filter_map(|r| r.ok())
                                                          .collect();
    // auto-fix a missing 'tile_cache' postfix on the input path -- easy to do when copy-pasting a
    // path to a wr-capture; there should at least be a frame00000.ron...
    let frame00000 = entries.iter().find(|&entry| entry.path().ends_with("frame00000.ron"));
    // ... and if not, try again with 'tile_cache' appended to the input folder
    if frame00000.is_none() {
        let new_path = input_dir.join("tile_cache");
        entries = std::fs::read_dir(new_path).unwrap()
                                             .filter_map(|r| r.ok())
                                             .collect();
    }
    entries.sort_by_key(|dir| dir.path());

    let mut svg_files: Vec::<String> = Vec::new();
    let mut intern_files: Vec::<String> = Vec::new();
    let mut prev_slices = None;

    let mut itemuid_to_string = HashMap::default();

    for entry in &entries {
        if entry.path().is_dir() {
            continue;
        }
        print!("processing {:?}\t", entry.path());
        let file_data = std::fs::read_to_string(entry.path()).unwrap();
        let chunks: Vec<_> = file_data.split("// @@@ chunk @@@").collect();
        let slices: Vec<Slice> = match ron::de::from_str(&chunks[0]) {
            Ok(data) => { data }
            Err(e) => {
                println!("ERROR: failed to deserialize slicesg {:?}\n{:?}", entry.path(), e);
                prev_slices = None;
                continue;
            }
        };
        let mut update_lists = TileCacheLoggerUpdateLists::new();
        update_lists.from_ron(&chunks[1]);
        update_lists.insert_in_lookup(&mut itemuid_to_string);

        let invalidation_report = write_tile_cache_visualizer_svg(
                                    &entry, &output_dir,
                                    &slices, prev_slices,
                                    &itemuid_to_string,
                                    &mut svg_width, &mut svg_height,
                                    &mut max_slice_index,
                                    &mut svg_files,
                                    &svg_settings);

        write_update_list_html(&entry, &output_dir, &update_lists,
                               &mut intern_files, invalidation_report);

        print!("\r");
        prev_slices = Some(slices);
    }

    write_html(output_dir, max_slice_index, &svg_files, &intern_files);
    write_css(output_dir, max_slice_index, &svg_settings);

    std::fs::write(output_dir.join("tilecache.js"), RES_JAVASCRIPT).unwrap();
    std::fs::write(output_dir.join("tilecache_base.css"), RES_BASE_CSS).unwrap();

    println!("\n");
}
