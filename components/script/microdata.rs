/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DocumentBinding::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::ElementBinding::ElementBinding::ElementMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::root::DomRoot;
use dom::document::Document;
use dom::element::Element;
use dom::htmlelement::HTMLElement;
use dom::node::Node;
use std::borrow::Cow;
use std::collections::HashMap;

pub struct Microdata {}

impl Microdata {
    //[Pref="dom.microdata.testing.enabled"]
    pub fn parse(doc: &Document) -> String {

        let ele = doc.upcast::<Node>();
        let mut start_vcard = false;
        let mut result : String = String::new();
        let mut master_map : HashMap<String, HashMap<String, String>> = HashMap::new();
        let mut master_key : String = String::new();

        result += "BEGIN:VCARD\nPROFILE:VCARD\nVERSION:4.0\nSOURCE:";
        result += doc.url().as_str();
        result += "\nNAME:";
        result += doc.Title().trim();
        result += "\n";

        for element in ele.traverse_preorder().filter_map(DomRoot::downcast::<Element>){
            if element.is::<HTMLElement>() {
                if element.has_attribute(&local_name!("itemtype")){
                    let mut atoms = element.get_tokenlist_attribute(&local_name!("itemtype"), );
                    if !atoms.is_empty() {
                        let val = atoms.remove(0);
                        if val.trim() == "http://microformats.org/profile/hcard"{
                            if !start_vcard {
                                start_vcard = true;
                            } else {
                                break;
                            }
                        }
                    }
                }
                if start_vcard {
                    let mut atoms = element.get_tokenlist_attribute(&local_name!("itemprop"), );
                    if !atoms.is_empty() {
                        let temp_key = atoms.remove(0);
                        if element.has_attribute(&local_name!("itemscope")){
                            master_key = String::from(temp_key.trim()).to_owned();
                            let dup_master_key = Cow::Borrowed(&master_key);
                            master_map.entry(dup_master_key.to_string()).or_insert(HashMap::new());
                        } else {
                            let temp = String::from(temp_key.trim()).to_owned();
                            let dup_key = Cow::Borrowed(&temp);
                            let data = String::from(element.GetInnerHTML().unwrap());
                            let dup_master_key = Cow::Borrowed(&master_key);
                            let temp_map = master_map.entry(dup_master_key.to_string()).or_insert(HashMap::new());
                            temp_map.insert(dup_key.to_string(), String::from(data));
                        }
                    }
                }
            }
        }
        //println!("{:?}", master_map);
        for (info_type, detail_map) in &master_map {
            //println!("{} -> {:?}", info_type, detail_map);
            match info_type.as_str() {
                "n" => {
                    let mut given_name = "";
                    let mut family_name = "";
                    if detail_map.contains_key("family-name") {
                        family_name = detail_map.get("family-name").unwrap();
                    }
                    if detail_map.contains_key("given-name") {
                        given_name = detail_map.get("given-name").unwrap();
                    }
                    result += format!("N:{};{}\n", family_name, given_name).as_str();
                    result += format!("FN:{} {}\n", given_name, family_name).as_str();
                },
                "org" => {
                    let mut organization_unit = "";
                    let mut organization_name = "";
                    if detail_map.contains_key("organization-unit"){
                        organization_unit = detail_map.get("organization-unit").unwrap();
                    }
                    if detail_map.contains_key("organization-name"){
                        organization_name = detail_map.get("organization-name").unwrap();
                    }
                    result += format!("ORG:{};{}\n", organization_name, organization_unit).as_str();
                },
                "tel" => {
                    
                },
                "adr" => {

                },
                _ => {},
            }
        }
        result += "END:VCARD";
        println!("{}", result);
        return result;
    }
}
