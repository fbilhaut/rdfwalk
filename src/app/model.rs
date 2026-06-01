use oxrdf::{Literal, NamedNode, Term};


#[derive(Debug, Clone)]
pub struct BrowserData {
    pub uri: NamedNode,
}

// Flat list of navigable items in the browser view
#[derive(Debug, Clone)]
pub enum BrowserItem {
    LiteralProp { prop: NamedNode, value: Literal },
    OutgoingLink { prop: NamedNode, target: NamedNode },
    IncomingLink { prop: NamedNode, source: NamedNode },
    AsPredicateRow { subject: NamedNode, object: Term },
}

impl BrowserItem {
    pub fn navigable_node(&self) -> Option<&NamedNode> {
        match self {
            BrowserItem::OutgoingLink { target, .. } => Some(target),
            BrowserItem::IncomingLink { source, .. } => Some(source),
            BrowserItem::AsPredicateRow { subject, .. } => Some(subject),
            BrowserItem::LiteralProp { .. } => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SparqlResult {
    pub variables: Vec<String>,
    pub rows: Vec<Vec<Option<Term>>>,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub resource: NamedNode,
    pub property: NamedNode,
    pub matched_value: String,
}