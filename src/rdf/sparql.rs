use anyhow::{Context, Result};
use oxrdf::{Literal, NamedNode, Term};
use sparesults::{QueryResultsFormat, QueryResultsParser, ReaderQueryResultsParserOutput};

pub struct QueryResult {
    pub variables: Vec<String>,
    pub rows: Vec<Vec<Option<Term>>>,
}

const LIMIT: usize = 1000; // TODO: pagination

pub struct SparqlClient {
    endpoint: String,
    client: reqwest::blocking::Client,
}

impl SparqlClient {
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            client: reqwest::blocking::Client::new(),
        }
    }

    fn query_raw(&self, sparql: &str) -> Result<Vec<u8>> {
        let response = self
            .client
            .get(&self.endpoint)
            .header("Accept", "application/sparql-results+xml")
            .query(&[("query", sparql)])
            .send()
            .context("HTTP request failed")?;
        let status = response.status();
        let bytes = response.bytes().context("reading response body")?;
        if !status.is_success() {
            let preview = String::from_utf8_lossy(&bytes[..bytes.len().min(300)]);
            anyhow::bail!("HTTP {} — {}", status, preview.trim());
        }
        Ok(bytes.to_vec())
    }

    fn parse_solutions<F, T>(bytes: Vec<u8>, mut f: F) -> Result<Vec<T>>
    where
        F: FnMut(sparesults::QuerySolution) -> Option<T>,
    {
        let parser = QueryResultsParser::from_format(QueryResultsFormat::Xml);
        let output = parser.for_reader(bytes.as_slice()).with_context(|| {
            let preview = String::from_utf8_lossy(&bytes[..bytes.len().min(200)]);
            format!("parsing SPARQL XML results (got: {})", preview.trim())
        })?;
        let mut result = Vec::new();
        if let ReaderQueryResultsParserOutput::Solutions(solutions) = output {
            for solution in solutions {
                let sol = solution.context("reading solution")?;
                if let Some(item) = f(sol) {
                    result.push(item);
                }
            }
        }
        Ok(result)
    }

    pub fn literal_properties(&self, uri: &NamedNode) -> Result<Vec<(NamedNode, Literal)>> {
        let q = format!(
            "SELECT ?p ?o WHERE {{ <{}> ?p ?o . FILTER(isLiteral(?o)) }} LIMIT {LIMIT}",
            uri.as_str()
        );
        let bytes = self.query_raw(&q)?;
        Self::parse_solutions(bytes, |sol| {
            let p = sol.get("p").cloned();
            let o = sol.get("o").cloned();
            match (p, o) {
                (Some(Term::NamedNode(p)), Some(Term::Literal(o))) => Some((p, o)),
                _ => None,
            }
        })
    }

    pub fn outgoing_links(&self, uri: &NamedNode) -> Result<Vec<(NamedNode, NamedNode)>> {
        let q = format!(
            "SELECT ?p ?o WHERE {{ <{}> ?p ?o . FILTER(isIRI(?o)) }} LIMIT {LIMIT}",
            uri.as_str()
        );
        let bytes = self.query_raw(&q)?;
        Self::parse_solutions(bytes, |sol| {
            let p = sol.get("p").cloned();
            let o = sol.get("o").cloned();
            match (p, o) {
                (Some(Term::NamedNode(p)), Some(Term::NamedNode(o))) => Some((p, o)),
                _ => None,
            }
        })
    }

    pub fn incoming_links(&self, uri: &NamedNode) -> Result<Vec<(NamedNode, NamedNode)>> {
        let q = format!(
            "SELECT ?s ?p WHERE {{ ?s ?p <{}> . FILTER(isIRI(?s)) }} LIMIT {LIMIT}",
            uri.as_str()
        );
        let bytes = self.query_raw(&q)?;
        Self::parse_solutions(bytes, |sol| {
            let s = sol.get("s").cloned();
            let p = sol.get("p").cloned();
            match (s, p) {
                (Some(Term::NamedNode(s)), Some(Term::NamedNode(p))) => Some((p, s)),
                _ => None,
            }
        })
    }

    pub fn as_predicate(&self, uri: &NamedNode) -> Result<Vec<(NamedNode, Term)>> {
        let q = format!(
            "SELECT ?s ?o WHERE {{ ?s <{}> ?o . FILTER(isIRI(?s)) }} LIMIT {LIMIT}",
            uri.as_str()
        );
        let bytes = self.query_raw(&q)?;
        Self::parse_solutions(bytes, |sol| {
            let s = sol.get("s").cloned();
            let o = sol.get("o").cloned();
            match (s, o) {
                (Some(Term::NamedNode(s)), Some(o)) => Some((s, o)),
                _ => None,
            }
        })
    }

    pub fn all_types(&self) -> Result<Vec<NamedNode>> {
        let q = format!("SELECT DISTINCT ?x WHERE {{ ?s <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> ?x . FILTER(isIRI(?x)) }} ORDER BY ?x LIMIT {LIMIT}");
        let bytes = self.query_raw(&q)?;
        Self::parse_solutions(bytes, |sol| {
            match sol.get("x").cloned() {
                Some(Term::NamedNode(n)) => Some(n),
                _ => None,
            }
        })
    }

    pub fn label_for(&self, uri: &NamedNode) -> Result<Option<String>> {
        let q = format!(
            "SELECT ?l WHERE {{ <{}> <http://www.w3.org/2000/01/rdf-schema#label> ?l }} LIMIT 1",
            uri.as_str()
        );
        let bytes = self.query_raw(&q)?;
        Ok(Self::parse_solutions(bytes, |sol| {
            match sol.get("l").cloned() {
                Some(Term::Literal(l)) => Some(l.value().to_string()),
                _ => None,
            }
        })?.into_iter().next())
    }

    pub fn run_query(&self, sparql: &str) -> Result<QueryResult> {
        let bytes = self.query_raw(sparql)?;
        let parser = QueryResultsParser::from_format(QueryResultsFormat::Xml);
        let output = parser.for_reader(bytes.as_slice()).with_context(|| {
            let preview = String::from_utf8_lossy(&bytes[..bytes.len().min(200)]);
            format!("parsing SPARQL XML results (got: {})", preview.trim())
        })?;
        match output {
            ReaderQueryResultsParserOutput::Solutions(solutions) => {
                let variables: Vec<String> = solutions.variables().iter()
                    .map(|v| v.as_str().to_string())
                    .collect();
                let mut rows: Vec<Vec<Option<Term>>> = Vec::new();
                for sol in solutions {
                    let sol = sol.context("reading solution")?;
                    rows.push(sol.values().iter().cloned().collect());
                }
                Ok(QueryResult { variables, rows })
            }
            ReaderQueryResultsParserOutput::Boolean(_) => {
                Ok(QueryResult { variables: vec![], rows: vec![] })
            }
        }
    }

    pub fn search_resources(&self, term: &str) -> Result<Vec<(NamedNode, NamedNode, String)>> {
        let escaped = term.replace('\\', "\\\\").replace('"', "\\\"");
        let q = format!(
            "SELECT DISTINCT ?s ?p ?o WHERE {{ \
             ?s ?p ?o . \
             FILTER(isLiteral(?o) && CONTAINS(LCASE(STR(?o)), LCASE(\"{escaped}\"))) \
             }} LIMIT {LIMIT}"
        );
        let bytes = self.query_raw(&q)?;
        Self::parse_solutions(bytes, |sol| {
            match (sol.get("s").cloned(), sol.get("p").cloned(), sol.get("o").cloned()) {
                (Some(Term::NamedNode(s)), Some(Term::NamedNode(p)), Some(Term::Literal(o))) => {
                    Some((s, p, o.value().to_string()))
                }
                _ => None,
            }
        })
    }
}
