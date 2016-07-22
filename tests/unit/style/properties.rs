/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::Au;
use cssparser::ToCss;
use rustc_serialize::json::Json;
use std::env;
use std::fs::{File, remove_file};
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use style::computed_values::display::T::inline_block;
use style::properties::longhands::border_top_width;
use style::properties::{DeclaredValue, PropertyDeclaration, PropertyDeclarationBlock};
use style::values::HasViewportPercentage;
use style::values::specified::{Length, LengthOrPercentageOrAuto, LengthOrPercentage, ViewportPercentageLength};

#[test]
fn properties_list_json() {
    let top = Path::new(file!()).parent().unwrap().join("..").join("..").join("..");
    let json = top.join("target").join("doc").join("servo").join("css-properties.json");
    if json.exists() {
        remove_file(&json).unwrap()
    }
    let python = env::var("PYTHON").ok().unwrap_or_else(find_python);
    let script = top.join("components").join("style").join("properties").join("build.py");
    let status = Command::new(python)
        .arg(&script)
        .arg("servo")
        .arg("html")
        .status()
        .unwrap();
    assert!(status.success());
    let properties = Json::from_reader(&mut File::open(json).unwrap()).unwrap();
    assert!(properties.as_object().unwrap().len() > 100);
    assert!(properties.find("margin").is_some());
    assert!(properties.find("margin-top").is_some());
}

#[cfg(windows)]
fn find_python() -> String {
    if Command::new("python27.exe").arg("--version").output().is_ok() {
        return "python27.exe".to_owned();
    }

    if Command::new("python.exe").arg("--version").output().is_ok() {
        return "python.exe".to_owned();
    }

    panic!("Can't find python (tried python27.exe and python.exe)! Try fixing PATH or setting the PYTHON env var");
}

#[cfg(not(windows))]
fn find_python() -> String {
    if Command::new("python2.7").arg("--version").output().unwrap().status.success() {
        "python2.7"
    } else {
        "python"
    }.to_owned()
}

#[test]
fn property_declaration_block_should_serialize_correctly() {
    let mut normal = Vec::new();
    let mut important = Vec::new();

    let length = LengthOrPercentageOrAuto::Length(Length::from_px(70f32));
    let value = DeclaredValue::Value(length);
    normal.push(PropertyDeclaration::Width(value));

    let min_height = LengthOrPercentage::Length(Length::from_px(20f32));
    let value = DeclaredValue::Value(min_height);
    normal.push(PropertyDeclaration::MinHeight(value));

    let value = DeclaredValue::Value(inline_block);
    normal.push(PropertyDeclaration::Display(value));

    let height = LengthOrPercentageOrAuto::Length(Length::from_px(20f32));
    let value = DeclaredValue::Value(height);
    important.push(PropertyDeclaration::Height(value));

    normal.reverse();
    important.reverse();
    let block = PropertyDeclarationBlock {
        normal: Arc::new(normal),
        important: Arc::new(important)
    };

    let css_string = block.to_css_string();

    assert_eq!(
        css_string,
        "width: 70px; min-height: 20px; display: inline-block; height: 20px !important;"
    );
}

#[test]
fn has_viewport_percentage_for_specified_value() {
    //TODO: test all specified value with a HasViewportPercentage impl
    let pvw = PropertyDeclaration::BorderTopWidth(
                  DeclaredValue::Value(border_top_width::SpecifiedValue(
                      Length::ViewportPercentage(ViewportPercentageLength::Vw(100.))
                  ))
              );
    assert!(pvw.has_viewport_percentage());

    let pabs = PropertyDeclaration::BorderTopWidth(
                   DeclaredValue::Value(border_top_width::SpecifiedValue(
                       Length::Absolute(Au(100))
                   ))
               );
    assert!(!pabs.has_viewport_percentage());
}
