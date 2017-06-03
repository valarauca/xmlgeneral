
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

/// An XML Item.
///
/// Currently this ignores namespaces.
#[derive(Clone,Debug)]
pub struct XmlItem {
  pub name: String,
  pub data: String,
  pub attributes: HashMap<String,String>,
  pub children: Vec<XmlItem>
}
impl Default for XmlItem {
  fn default() -> Self {
    XmlItem {
      name: String::with_capacity(0),
      data: String::with_capacity(0),
      attributes: HashMap::new(),
      children: Vec::with_capacity(0)
    }
  }
}
impl XmlItem {
  fn assign_name(&mut self, name: String) {
    self.name = name;
  }
  fn assign_data(&mut self, data: String) {
    self.data = data;
  }
  ///Duplicate names are removed
  fn assign_attributes(&mut self, attr: &[OwnedAttribute]) {
    for pair in attr.iter() {
      if self.attributes.contains_key(&pair.name.local_name) {
        continue;
      }
      self.attributes.insert(pair.name.local_name.clone(), pair.value.clone());
    }
  }
}

/// This is called while building the body of an item
fn recursively_build<R: Read>(events: &mut EventReader<R>, item: &mut XmlItem) -> Result<(),String> {
  loop {
    let event = events.next();
    match event {
      Err(e) => return Err(format!("Parser error {:?}", e)),
      Ok(XmlEvent::StartDocument{ version, encoding, standalone}) => return Err(format!("Encountered the start of the document")),
      Ok(XmlEvent::EndDocument) => return Err(format!("Encountered the end of the document")),
      Ok(XmlEvent::ProcessingInstruction{ name, data}) => continue,
      Ok(XmlEvent::CData(_)) |
      Ok(XmlEvent::Comment(_)) |
      Ok(XmlEvent::Whitespace(_)) => continue,
      Ok(XmlEvent::StartElement{ name, attributes, namespace }) => {
        let mut local = XmlItem::default();
        local.assign_name(name.local_name);
        local.assign_attributes(attributes.as_slice());
        match recursively_build(events, &mut local) {
          Ok(()) => { },
          Err(e) => return Err(e)
        };
        item.children.push(local);
      },
      Ok(XmlEvent::Characters(ref string)) => {
        item.data.push_str(string);
      },
      Ok(XmlEvent::EndElement{ name }) => {
        if name.local_name == item.name {
          return Ok(());
        } else {
          return Err(format!("Expected end of {:?} found end of {:?}", &item.name, &name.local_name));
        }
      }
    };
  }
}

/// Reads XML Item
pub fn read_xml<R: Read>(doc: R) -> Result<Vec<XmlItem>, String> {
  let mut output = Vec::<XmlItem>::new();
  let mut events = EventReader::new(doc);
  loop {
    let event = events.next();
    match event {
      Err(e) => return Err(format!("Parser error {:?}", e)),
      Ok(XmlEvent::StartDocument{ version, encoding, standalone}) => continue,
      Ok(XmlEvent::EndDocument) => return Ok(output),
      Ok(XmlEvent::ProcessingInstruction{ name, data}) => continue,
      Ok(XmlEvent::CData(_)) |
      Ok(XmlEvent::Comment(_)) |
      Ok(XmlEvent::Whitespace(_)) => continue,
      Ok(XmlEvent::StartElement{ name, attributes, namespace }) => {
        let mut local = XmlItem::default();
        local.assign_name(name.local_name);
        local.assign_attributes(attributes.as_slice());
        match recursively_build(&mut events, &mut local) {
          Ok(()) => { },
          Err(e) => return Err(e)
        };
        output.push(local);
      },
      Ok(XmlEvent::Characters(ref string)) => return Err(format!("Shouldn't encounter data directly")),
      Ok(XmlEvent::EndElement{ name }) => return Err(format!("End of elements should be handled with the recursive handler"))
    };
  }
}
