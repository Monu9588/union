use std::{env, fs};

use schemars::{
    schema::{InstanceType, RootSchema, Schema, SchemaObject, SingleOrVec},
    visit::Visitor,
};

fn main() {
    let mut root_schema = serde_json::from_str::<RootSchema>(
        &fs::read_to_string(env::args().nth(1).unwrap()).unwrap(),
    )
    .unwrap();

    dbg!(&root_schema);

    let mut visitor = JsonSchemaToNixosModuleOptions {
        output: String::new(),
        at_root: true,
    };

    visitor.visit_root_schema(&mut root_schema);

    println!("{}", visitor.output);
}

struct JsonSchemaToNixosModuleOptions {
    output: String,
    at_root: bool,
}

impl Visitor for JsonSchemaToNixosModuleOptions {
    fn visit_root_schema(&mut self, root: &mut RootSchema) {
        self.output += "{ types, mkOption }: let\ndefinitions = {";

        for (name, schema) in &mut root.definitions {
            self.output += &format!(r##""#/definitions/{name}" = "##);
            self.visit_schema(schema);
            self.output += ";\n";
        }

        self.output += "};\nin\n";

        self.at_root = false;

        self.visit_schema_object(&mut root.schema);

        // self.output += "";
    }

    fn visit_schema(&mut self, schema: &mut Schema) {
        match schema {
            Schema::Bool(_) => {
                self.output += "types.any";
            }
            Schema::Object(obj) => self.visit_schema_object(obj),
        }
    }

    fn visit_schema_object(&mut self, schema: &mut SchemaObject) {
        match (&mut schema.instance_type, &mut schema.reference) {
            (Some(instance_type), None) => match instance_type {
                SingleOrVec::Single(ty) => match &**ty {
                    InstanceType::Null => todo!(),
                    InstanceType::Boolean => {
                        self.output += "types.bool";
                    }
                    InstanceType::Object => {
                        let obj_val = schema.object.as_mut().unwrap();
                        if self.at_root {
                            self.output += "{";
                        } else {
                            self.output += "types.submodule { options = {";
                        };
                        for (property_name, property) in &mut obj_val.properties {
                            self.output += &format!(r#""{property_name}" = mkOption {{ type = "#);
                            self.visit_schema(property);
                            self.output += ";};\n";
                        }
                        if self.at_root {
                            self.output += "}";
                        } else {
                            self.output += "};}";
                        };
                    }
                    InstanceType::Array => {
                        self.output += "types.listOf (";

                        match schema.array.as_mut().unwrap().items.as_mut().unwrap() {
                            SingleOrVec::Single(item_schema) => self.visit_schema(item_schema),
                            SingleOrVec::Vec(_) => todo!(),
                        };

                        self.output += ")";
                    }
                    InstanceType::Number => {
                        self.output += "types.number";
                    }
                    InstanceType::String => {
                        self.output += "types.str";
                    }
                    InstanceType::Integer => {
                        self.output += "types.int";
                    }
                },
                SingleOrVec::Vec(_) => todo!(),
            },
            (None, Some(reference)) => {
                self.output += "definitions.\"";
                self.output += reference;
                self.output += "\"";
            }
            (None, None) => {
                self.output += "types.attrs";
            }
            _ => {
                println!("{}", self.output);

                todo!("{schema:#?}");
            }
        }
    }
}
