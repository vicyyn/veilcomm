use crate::NodesTable;
use dioxus::prelude::*;

#[derive(PartialEq, Props)]
pub struct TableProps {
    nodes_table: NodesTable,
}

pub fn Table(cx: Scope<TableProps>) -> Element {
    cx.render(rsx!(
         table {
            tr {
                th { "IP" }
                th { "Port" }
                th { "Address" }
            }
            for (ip, port, address) in &cx.props.nodes_table.get_tuples() {
                tr {
                    td { ip.as_str() }
                    td { format!("{}", port) }
                    td { address.as_str() }
                }
            }
        }
    ))
}
