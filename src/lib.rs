
extern crate xml;
use xml::reader::{
  Error as XmlError,
  EventReader,
  XmlEvent
};
use xml::attribute::OwnedAttribute;

use std::io::Read;
use std::default::Default;
use std::collections::HashMap;



///Get all events
///
///Reads all events into memory so the structure of the document
///just consists of pointers to that document
pub fn get_events<R: Read>(doc: R) -> Result<Vec<XmlEvent>, String> {
    let mut retvec = Vec::new();
    for event in EventReader::new(doc).into_iter() {
        match event {
            Ok(x) => retvec.push(x),
            Err(e) => return Err(format!("{:?}",e))
        };
    }
    Ok(retvec)
}

/// An XML Item.
///
/// Currently this ignores namespaces.
///
/// Typically usage is a `get_events(Read)` into a `
#[derive(Clone,Debug)]
pub struct XmlItem<'a> {
    pub name: &'a str,
    pub data: &'a str,
    pub attributes: HashMap<&'a str, &'a str>,
    pub children: Vec<XmlItem<'a>>
}
impl<'a> XmlItem<'a> {

    fn make_hashmap(attr: &'a [OwnedAttribute]) -> HashMap<&'a str, &'a str> {
        let mut map = HashMap::new();
        for attribute in attr.iter() {
            map.insert(
                attribute.name.local_name.as_ref(),
                attribute.value.as_ref());
        }
        map
    }

    fn build<I: Iterator<Item=&'a XmlEvent>>(name: &'a str, attributes: &'a [OwnedAttribute], iter: &mut I) -> Result<XmlItem<'a>, String> {
        let mut item_name = name;
        let mut data: Option<&'a str> = None;
        let map = XmlItem::make_hashmap(attributes);
        let mut children = Vec::with_capacity(0);
        loop {
            let e = iter.next();    
            match e {
                Option::None => return Err(format!("Encountered end incorrectly")),
                Option::Some(&XmlEvent::StartDocument{ref version, ref encoding, ref standalone}) => return Err(format!("Encountered two start of documents")),
                Option::Some(&XmlEvent::EndDocument) => return Err(format!("Encountered end of document whole building an item")),
                Option::Some(&XmlEvent::ProcessingInstruction{ ref name, ref data }) => continue,
                Option::Some(&XmlEvent::CData(_)) |
                Option::Some(&XmlEvent::Comment(_)) |
                Option::Some(&XmlEvent::Whitespace(_)) => continue,
                Option::Some(&XmlEvent::Characters(ref string)) => {
                    let mut thing: Option<&'a str> = Some(string);
                    ::std::mem::swap(&mut thing, &mut data);
                },
                Option::Some(&XmlEvent::EndElement{ ref name }) => if item_name == name.local_name {
                        match data {
                            Option::None => return Ok(XmlItem {
                                name: &name.local_name, 
                                data: "", 
                                attributes: map, 
                                children: children 
                            }),
                            Option::Some(var) => return Ok(XmlItem { 
                                name: &name.local_name,
                                data: var, 
                                attributes: map, 
                                children: children 
                            })
                        };
                    } else {
                        return Err(format!("Encountered end of an item that never started"));
                },
                Option::Some(&XmlEvent::StartElement{ ref name, ref attributes, ref namespace }) => match XmlItem::build(&name.local_name, attributes, iter) {
                    Ok(x) => {
                        children.push(x);
                        continue;
                    },
                    Err(e) => return Err(e)
                }
            };
        }
    }
}

/// Reads XML Item
pub fn read_xml<'a>(events: &'a [XmlEvent]) -> Result<Vec<XmlItem<'a>>, String> {
    let mut event_iterator = events.iter();
    let mut ret_vec = Vec::new();
    loop {
        let event = event_iterator.next();
        match event {
            Option::None => return Err(format!("Countered NONE should encounter XmlItem::EndDocument to end gracefully")),
            Option::Some(&XmlEvent::StartDocument{ref version, ref encoding, ref standalone}) => continue,
            Option::Some(&XmlEvent::EndDocument) => return Ok(ret_vec),
            Option::Some(&XmlEvent::ProcessingInstruction{ref name, ref data}) => continue,
            Option::Some(&XmlEvent::CData(_)) => continue,
            Option::Some(&XmlEvent::Comment(_)) => continue,
            Option::Some(&XmlEvent::Whitespace(_)) => continue,
            Option::Some(&XmlEvent::StartElement{ref name, ref attributes, ref namespace}) => match XmlItem::build(&name.local_name, attributes, &mut event_iterator) {
                Ok(var) => ret_vec.push(var),
                Err(e) => return Err(e)
            },
            Option::Some(&XmlEvent::Characters(_)) => return Err(format!("Shouldn't encounter data directly")),
            Option::Some(&XmlEvent::EndElement{ref name}) => return Err(format!("End of elements should be handled with the recursive handler"))
        };
    }
}
