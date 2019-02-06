#[macro_use]
extern crate debug_stub_derive;

use once_cell::unsync::OnceCell;
use serde_derive::Deserialize;
use serde_json::{self, json};
use std::rc::{Rc, Weak};

type SchemaRef = Rc<Schema>;
type SchemaWeakRef = Weak<Schema>;

#[derive(Deserialize, Debug)]
struct SchemaTemplate {
    pub name: String,
    pub models: Vec<ModelTemplate>,
}

#[derive(Debug)]
struct Schema {
    pub name: String,
    pub models: OnceCell<Vec<Model>>,
}

#[derive(Deserialize, Debug)]
struct ModelTemplate {
    pub name: String,
}

#[derive(DebugStub)]
struct Model {
    pub name: String,
    #[debug_stub = "#SchemaWeakRef#"]
    pub schema: SchemaWeakRef,
}

impl ModelTemplate {
    pub fn build(self, schema: SchemaWeakRef) -> Model {
        Model {
            name: self.name,
            schema,
        }
    }
}

impl Into<SchemaRef> for SchemaTemplate {
    fn into(self) -> SchemaRef {
        let schema = Rc::new(Schema {
            name: self.name,
            models: OnceCell::new(),
        });

        let models = self
            .models
            .into_iter()
            .map(|mt| mt.build(Rc::downgrade(&schema)))
            .collect();

        // OnceCell is new here, containing no data and will not panic. Safe to
        // unwrap.
        schema.models.set(models).unwrap();

        schema
    }
}

impl Model {
    fn with_schema<F, T>(&self, f: F) -> T
    where
        F: FnOnce(Rc<Schema>) -> T,
    {
        match self.schema.upgrade() {
            Some(schema) => f(schema),
            None => {
                panic!("Parent Schema is dead and Model still exists, always delete them together!")
            }
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
        self.models
            .get()
            .and_then(|models| models.iter().find(|model| model.name == name))
    }
}

fn main() {
    let json = json!({
        "name": "test",
        "models": [{"name": "testo"}]
    });

    let template: SchemaTemplate = serde_json::from_value(json).unwrap();
    let schema: SchemaRef = template.into();
    let model = schema.find_model("testo").unwrap();

    model.print_schema();
    dbg!(schema);
}
