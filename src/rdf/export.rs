use anyhow::{Context, Result};
use oxrdf::{Term, Variable, VariableRef};
use sparesults::{QueryResultsFormat, QueryResultsSerializer};
use std::io::Write;

/// Serializes SPARQL query solutions using the standard SPARQL Query Results
/// formats (CSV, TSV, JSON or XML, all defined by W3C) provided by `sparesults`.
pub fn write_solutions<W: Write>(
    writer: W,
    format: QueryResultsFormat,
    variables: &[String],
    rows: &[Vec<Option<Term>>],
) -> Result<()> {
    let vars: Vec<Variable> = variables.iter()
        .map(|v| Variable::new(v.as_str()))
        .collect::<Result<_, _>>()
        .context("invalid SPARQL variable name")?;
    let mut solutions = QueryResultsSerializer::from_format(format)
        .serialize_solutions_to_writer(writer, vars.clone())?;
    let var_refs: Vec<VariableRef> = vars.iter().map(Variable::as_ref).collect();
    for row in rows {
        let bindings = var_refs.iter()
            .zip(row.iter())
            .filter_map(|(v, t)| t.as_ref().map(|term| (*v, term.as_ref())));
        solutions.serialize(bindings)?;
    }
    solutions.finish()?;
    Ok(())
}
