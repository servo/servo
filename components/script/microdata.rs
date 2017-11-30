/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DocumentBinding::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::ElementBinding::ElementBinding::ElementMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::root::DomRoot;
use dom::characterdata::CharacterData;
use dom::document::Document;
use dom::element::Element;
use dom::htmlelement::HTMLElement;
use dom::node::Node;
use dom::text::Text;
use std::borrow::Cow;
use std::collections::HashMap;
use serde_json;

pub struct Microdata {}
#[derive(Serialize,Clone)]
#[serde(untagged)]
enum Data {
    StrValue(String),
    VecValue(Vec<Data>),
    DataValue(Box<Data>),
    HashValue(HashMap<String,Data>)
}

impl Microdata {
    //[Pref="dom.microdata.testing.enabled"]
    pub fn parse(doc: &Document, node : &Node) -> HashMap<String, String> {
        //let dup_doc = Cow::Borrowed(doc);
        let serialized_vcard = Self::parse_vcard(doc);
        let serialized_json = Self::parse_json(node);
        let mut serialized_data : HashMap<String, String> = HashMap::new();
        serialized_data.insert("vcard".to_string(), serialized_vcard);
        serialized_data.insert("json".to_string(), serialized_json);
        return serialized_data;
    }

    pub fn parse_vcard(doc: &Document) -> String {

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

        for (info_type, detail_map) in &master_map {

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

    pub fn parse_json(node: &Node) -> String {
        let json_data : Data = Self::traverse(node).unwrap();
        let json = serde_json::to_string(&json_data);
        //println!("printing json from microdata {:?}", json);
        return json.ok().unwrap();
    }

    fn get_attr_value(element: &Element, property: &str)-> Option<String> {
        println!("{:?}",property);
        // let mut atoms =  match property {
        //   "itemprop" => element.get_tokenlist_attribute(&local_name!("itemprop"), ),
        //   "itemtype" =>  element.get_tokenlist_attribute(&local_name!("itemtype"), ),
        //   _ => {},
        // };
        // if !atoms.is_empty() {
        //   let temp_key = atoms.remove(0);
        //   return Some(String::from(temp_key.trim()).to_owned());
        // }
        // else {
        //   return None;
        // }
        Some(String::from("itemprop"))
    }

    fn traverse( node: &Node)-> Option<Data> {
        if !node.is::<Element>(){
            if let Some(ref text) = node.downcast::<Text>(){
                let mut content = String::new();
                content.push_str(&text.upcast::<CharacterData>().data());
                return Some(Data::StrValue(String::from(content)));
            }
            None
        }
        else {
            let element = node.downcast::<Element>().unwrap();
            let mut head_str = String::from("");
            let mut parent_vec:Vec<Data> = Vec::new();
            let item_type = Self::get_attr_value(element,"itemtype").unwrap();
            // if element.has_attribute(&local_name!("itemscope")) && element.has_attribute(&local_name!("itemtype")) && !element.has_attribute(&local_name!("itemprop")) {
            //     head_str = String::from("items");
            //     let mut propMap:HashMap<String,Data> = HashMap::new();
            //     //Data::HashValue(propMap)
            //     let item_type = Self::get_attr_value(element,"item_type").unwrap();

            // }
            // else if element.has_attribute(&local_name!("itemprop")) && element.has_attribute(&local_name!("item_type")) {

            //     head_str = Self::get_attr_value(element,"itemprop").unwrap();
            //     let item_type = Self::get_attr_value(element,"item_type").unwrap();

            // }
            // else {
            //     return None;
            // }
            let mut inner_map:HashMap<String,Data> = HashMap::new();
            for child in node.children(){
                if let Some(childData) = Self::traverse(child.upcast::<Node>()){
                    parent_vec.push(childData);
                }
            }
            inner_map.insert(head_str,Data::VecValue(parent_vec));
            Some(Data::HashValue(inner_map))
        }
    }
}
