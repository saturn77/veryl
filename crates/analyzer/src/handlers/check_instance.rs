use crate::analyzer_error::AnalyzerError;
use crate::symbol::SymbolKind;
use crate::symbol_table;
use veryl_parser::resource_table;
use veryl_parser::veryl_grammar_trait::*;
use veryl_parser::veryl_walker::{Handler, HandlerPoint};
use veryl_parser::ParolError;

pub struct CheckInstance<'a> {
    pub errors: Vec<AnalyzerError>,
    text: &'a str,
    point: HandlerPoint,
}

impl<'a> CheckInstance<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            errors: Vec::new(),
            text,
            point: HandlerPoint::Before,
        }
    }
}

impl<'a> Handler for CheckInstance<'a> {
    fn set_point(&mut self, p: HandlerPoint) {
        self.point = p;
    }
}

impl<'a> VerylGrammarTrait for CheckInstance<'a> {
    fn inst_declaration(&mut self, arg: &InstDeclaration) -> Result<(), ParolError> {
        if let HandlerPoint::Before = self.point {
            let mut connected_ports = Vec::new();
            if let Some(ref x) = arg.inst_declaration_opt1 {
                if let Some(ref x) = x.inst_declaration_opt2 {
                    let x = &x.inst_port_list;
                    connected_ports.push(x.inst_port_item.identifier.identifier_token.token.text);
                    for x in &x.inst_port_list_list {
                        connected_ports
                            .push(x.inst_port_item.identifier.identifier_token.token.text);
                    }
                }
            }

            if let Ok(symbol) = symbol_table::resolve(arg.identifier0.as_ref()) {
                let name = arg.identifier0.identifier_token.text();
                if let Some(symbol) = symbol.found {
                    match symbol.kind {
                        SymbolKind::Module(ref x) => {
                            for port in &x.ports {
                                if !connected_ports.contains(&port.name) {
                                    let port = resource_table::get_str_value(port.name).unwrap();
                                    self.errors.push(AnalyzerError::missing_port(
                                        &name,
                                        &port,
                                        self.text,
                                        &arg.identifier.identifier_token,
                                    ));
                                }
                            }
                            for port in &connected_ports {
                                if !x.ports.iter().any(|x| &x.name == port) {
                                    let port = resource_table::get_str_value(*port).unwrap();
                                    self.errors.push(AnalyzerError::unknown_port(
                                        &name,
                                        &port,
                                        self.text,
                                        &arg.identifier.identifier_token,
                                    ));
                                }
                            }
                        }
                        SymbolKind::Interface(_) => (),
                        _ => {
                            self.errors.push(AnalyzerError::mismatch_type(
                                &name,
                                "module or interface",
                                &symbol.kind.to_kind_name(),
                                self.text,
                                &arg.identifier.identifier_token,
                            ));
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
