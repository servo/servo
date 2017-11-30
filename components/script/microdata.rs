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

        let title = doc.Title();
        if !title.is_empty() && !title.trim().is_empty() {
            result += "\nNAME:";
            result += title.trim();
        }

        result += "\n";
        let mut sex : String = String::new();
        let mut gender_identity : String = String::new();

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

                    let mut n_value : String = String::new();

                    let name_parts = ["family-name", "given-name", "additional-name", "honorific-prefix", "honorific-suffix"];
                    for part in name_parts.iter() {
                        if detail_map.contains_key(*part) {
                            n_value += format!("{};",detail_map.get(*part).unwrap()).as_str();
                        }
                    }
                    n_value.pop();

                    result += format!("{}:{}\n", info_type.as_str(), n_value).as_str();

                },
                "org" => {

                    let mut org_value : String = String::new();

                    let org_parts = ["organization-name", "organization-unit"];
                    for part in org_parts.iter() {
                        if detail_map.contains_key(*part) {
                            org_value += format!("{};",detail_map.get(*part).unwrap()).as_str();
                        }
                    }
                    org_value.pop();

                    result += format!("{}:{}\n", info_type.as_str(), org_value).as_str();

                },
                "tel" => {

                },
                "adr" => {

                    let mut adr_value : String = String::new();

                    let adr_parts = ["street-address", "locality", "region", "postal-code", "country-name", "post-office-box", "extended-address"];
                    for part in adr_parts.iter() {
                        if detail_map.contains_key(*part) {
                            adr_value += format!("{};",detail_map.get(*part).unwrap()).as_str();
                        }
                    }
                    adr_value.pop();

                    result += format!("{}:{}\n", info_type.as_str(), adr_value).as_str();

                },
                _ => {},
            }
        }

        result += "END:VCARD";
        println!("{}", result);
        return result;
    }
}
