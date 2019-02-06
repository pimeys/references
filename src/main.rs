#[macro_use]
extern crate debug_stub_derive;

use serde_derive::Deserialize;
use serde_json::{json, self};
use std::{rc::{Weak, Rc}, cell::{Ref, RefCell}};

type SchemaRef = Rc<RefCell<Schema>>;
type SchemaWeakRef = Weak<RefCell<Schema>>;

#[derive(Deserialize, Debug)]
struct SchemaTemplate {
    pub name: String,
    pub models: Vec<ModelTemplate>
}

#[derive(Debug)]
struct Schema {
    pub name: String,
    pub models: Vec<Model>,
}

#[derive(Deserialize, Debug)]
struct ModelTemplate {
    pub name: String
}

#[derive(DebugStub)]
struct Model {
    pub name: String,
    #[debug_stub="#SchemaWeakRef#"]
    pub schema: SchemaWeakRef,
}

impl ModelTemplate {
    pub fn build(self, schema: SchemaWeakRef) -> Model {
        Model {
            name: self.name,
            schema: schema,
        }
    }
}

impl Into<SchemaRef> for SchemaTemplate {
    fn into(self) -> SchemaRef {
        let schema = Rc::new(RefCell::new(Schema {
            name: self.name,
            models: Vec::new(),
        }));

        self.models.into_iter().for_each(|mt| {
            schema.borrow_mut().models.push(mt.build(Rc::downgrade(&schema)));
        });
        
        schema
    }
}

impl Model {
    fn with_schema<F, T>(&self, f: F) -> T
    where
        F: FnOnce(Ref<Schema>) -> T
    {
        match self.schema.upgrade() {
            Some(schema) => f(schema.borrow()),
            None => panic!("Parent Schema is dead and Model still exists, always delete them together!"),
        }
    }

    pub fn print_schema(&self) {
        self.with_schema(|schema| {
            println!("{}", schema.name);
        })
    }
}

impl Schema {
    pub fn find_model(&self, name: &str) -> Option<&Model> {
        self.models.iter().find(|model| model.name == name)
    }
}

fn main() {
    let json = json!({
        "name": "test",
        "models": [{"name": "testo"}]
    });
    
    let template: SchemaTemplate = serde_json::from_value(json).unwrap();
    let schema: SchemaRef = template.into();
    let schema_borrow = schema.borrow();
    let model = schema_borrow.find_model("testo").unwrap();
    
    model.print_schema();
    dbg!(schema_borrow);
}
