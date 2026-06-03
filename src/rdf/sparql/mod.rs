use anyhow::Result;
use oxrdf::{Literal, NamedNode, Term};

mod remote;
#[cfg(feature = "local")] mod local;

pub struct QueryResult {
    pub variables: Vec<String>,
    pub rows: Vec<Vec<Option<Term>>>,
}

impl QueryResult {
    fn get_var<'a>(&self, row: &'a [Option<Term>], var: &str) -> Option<&'a Term> {
        let idx = self.variables.iter().position(|v| v == var)?;
        row.get(idx)?.as_ref()
    }
}

// Backend Trait
pub trait SparqlBackend: Send + Sync {
    fn run_query(&self, sparql: &str) -> Result<QueryResult>;
}


// Public facade
pub struct SparqlClient {
    backend: Box<dyn SparqlBackend>,
    limit: usize,
}

impl SparqlClient {
    pub fn remote(endpoint: String) -> Self {
        Self { backend: Box::new(remote::RemoteBackend::new(endpoint)), limit: 1000 }
    }

    #[cfg(feature = "local")]
    pub fn local(path: &str) -> Result<Self> {
        Ok(Self { backend: Box::new(local::LocalBackend::from_file(path)?), limit: 1000 })
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    pub fn literal_properties(&self, uri: &NamedNode) -> Result<Vec<(NamedNode, Literal)>> {
        let limit = self.limit;
        let q = format!(
            "SELECT ?p ?o WHERE {{ <{}> ?p ?o . FILTER(isLiteral(?o)) }} LIMIT {limit}",
            uri.as_str()
        );
        let result = self.backend.run_query(&q)?;
        Ok(result.rows.iter().filter_map(|row| {
            match (result.get_var(row, "p").cloned(), result.get_var(row, "o").cloned()) {
                (Some(Term::NamedNode(p)), Some(Term::Literal(o))) => Some((p, o)),
                _ => None,
            }
        }).collect())
    }

    pub fn outgoing_links(&self, uri: &NamedNode) -> Result<Vec<(NamedNode, NamedNode)>> {
        let limit = self.limit;
        let q = format!(
            "SELECT ?p ?o WHERE {{ <{}> ?p ?o . FILTER(isIRI(?o)) }} LIMIT {limit}",
            uri.as_str()
        );
        let result = self.backend.run_query(&q)?;
        Ok(result.rows.iter().filter_map(|row| {
            match (result.get_var(row, "p").cloned(), result.get_var(row, "o").cloned()) {
                (Some(Term::NamedNode(p)), Some(Term::NamedNode(o))) => Some((p, o)),
                _ => None,
            }
        }).collect())
    }

    pub fn incoming_links(&self, uri: &NamedNode) -> Result<Vec<(NamedNode, NamedNode)>> {
        let limit = self.limit;
        let q = format!(
            "SELECT ?s ?p WHERE {{ ?s ?p <{}> . FILTER(isIRI(?s)) }} LIMIT {limit}",
            uri.as_str()
        );
        let result = self.backend.run_query(&q)?;
        Ok(result.rows.iter().filter_map(|row| {
            match (result.get_var(row, "s").cloned(), result.get_var(row, "p").cloned()) {
                (Some(Term::NamedNode(s)), Some(Term::NamedNode(p))) => Some((p, s)),
                _ => None,
            }
        }).collect())
    }

    pub fn as_predicate(&self, uri: &NamedNode) -> Result<Vec<(NamedNode, Term)>> {
        let limit = self.limit;
        let q = format!(
            "SELECT ?s ?o WHERE {{ ?s <{}> ?o . FILTER(isIRI(?s)) }} LIMIT {limit}",
            uri.as_str()
        );
        let result = self.backend.run_query(&q)?;
        Ok(result.rows.iter().filter_map(|row| {
            match (result.get_var(row, "s").cloned(), result.get_var(row, "o").cloned()) {
                (Some(Term::NamedNode(s)), Some(o)) => Some((s, o)),
                _ => None,
            }
        }).collect())
    }

    pub fn all_types(&self) -> Result<Vec<NamedNode>> {
        let limit = self.limit;
        let q = format!(
            "SELECT DISTINCT ?x WHERE {{ \
             ?s <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> ?x . \
             FILTER(isIRI(?x)) \
             }} ORDER BY ?x LIMIT {limit}"
        );
        let result = self.backend.run_query(&q)?;
        Ok(result.rows.iter().filter_map(|row| {
            match result.get_var(row, "x").cloned() {
                Some(Term::NamedNode(n)) => Some(n),
                _ => None,
            }
        }).collect())
    }

    pub fn label_for(&self, uri: &NamedNode) -> Result<Option<String>> {
        let q = format!(
            "SELECT ?l WHERE {{ \
             <{}> <http://www.w3.org/2000/01/rdf-schema#label> ?l \
             }} LIMIT 1",
            uri.as_str()
        );
        let result = self.backend.run_query(&q)?;
        Ok(result.rows.iter().find_map(|row| {
            match result.get_var(row, "l").cloned() {
                Some(Term::Literal(l)) => Some(l.value().to_string()),
                _ => None,
            }
        }))
    }

    pub fn run_query(&self, sparql: &str) -> Result<QueryResult> {
        self.backend.run_query(sparql)
    }

    pub fn search_resources(&self, term: &str) -> Result<Vec<(NamedNode, NamedNode, String)>> {
        let limit = self.limit;
        let escaped = term.replace('\\', "\\\\").replace('"', "\\\"");
        let q = format!(
            "SELECT DISTINCT ?s ?p ?o WHERE {{ \
             ?s ?p ?o . \
             FILTER(isLiteral(?o) && CONTAINS(LCASE(STR(?o)), LCASE(\"{escaped}\"))) \
             }} LIMIT {limit}"
        );
        let result = self.backend.run_query(&q)?;
        Ok(result.rows.iter().filter_map(|row| {
            match (
                result.get_var(row, "s").cloned(),
                result.get_var(row, "p").cloned(),
                result.get_var(row, "o").cloned(),
            ) {
                (Some(Term::NamedNode(s)), Some(Term::NamedNode(p)), Some(Term::Literal(o))) => {
                    Some((s, p, o.value().to_string()))
                }
                _ => None,
            }
        }).collect())
    }
}
