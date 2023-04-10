use dioxus::prelude::*;

pub mod components;
pub mod models;

pub use components::*;
pub use models::*;

fn app(cx: Scope) -> Element {
    let mut nodes_table = NodesTable::new();
    nodes_table.add_node_row(NodeRow::default());
    nodes_table.add_node_row(NodeRow::default());
    nodes_table.add_node_row(NodeRow::default());
    nodes_table.add_node_row(NodeRow::default());

    cx.render(rsx!(
    div {
        Table {
            nodes_table: nodes_table
    }}))
}

fn main() {
    dioxus_web::launch(app);
}
