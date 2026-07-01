# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

`rdfwalk` is a terminal UI (ratatui) for browsing RDF data, either through a remote SPARQL endpoint or an in-memory local RDF file. Single binary crate, edition 2024.

## Commands

```
cargo build                       # remote mode only (default features)
cargo build --features local      # includes local file browsing (spareval/spargebra/oxrdfio)
cargo run -- <endpoint> [start-uri]
cargo run --features local -- --local <file> [start-uri]
cargo clippy --features local     # lint with all features enabled to cover both backends
```

There is currently no test suite (`cargo test` finds nothing) — verify changes via `cargo build`/`cargo clippy` and, for behavior, by running the TUI.

## Architecture

### Two SPARQL backends behind one trait

`rdf/sparql/mod.rs` defines `SparqlBackend { fn run_query(&self, sparql: &str) -> Result<QueryResult> }` and a facade `SparqlClient` that builds ad-hoc SPARQL queries for every browser/type/search operation (`literal_properties`, `outgoing_links`, `incoming_links`, `as_predicate`, `all_types`, `label_for`, `search_resources`) and issues them through the trait object. There is no separate triple-store query layer — everything, including resource browsing, is expressed as SPARQL and routed through `run_query`.

- `rdf/sparql/remote.rs`: `reqwest::blocking` GET against the endpoint, `Accept: application/sparql-results+xml`, parsed with `sparesults::QueryResultsParser`.
- `rdf/sparql/local.rs` (feature `local`): parses the query with `spargebra`, evaluates in-memory with `spareval::QueryEvaluator` against an `ArcDataset` wrapper around `oxrdf::Dataset` (implements `spareval::QueryableDataset` by delegating to `Dataset`'s own impl, cloning only the `Arc`, not the quads).

Both backends normalize their output to the same shape: `QueryResult { variables: Vec<String>, rows: Vec<Vec<Option<Term>>> }`. Any new consumer of query results (e.g. export) should work off this shape rather than caring which backend produced it.

`--limit <n>` (default 1000) is threaded through `SparqlClient::with_limit` and appended as `LIMIT` to every generated query — it bounds all backend-generated queries, not just user-typed ones.

### App state is one flat struct, not per-view state machines

`app/mod.rs` holds a single `App` struct with all state for every view inlined as fields (`sparql_input`, `sparql_result`, `search_input`, `browser_selection`, etc.), and a `View` enum (`Browser`, `Types`, `Sparql`, `Search`, `Bookmarks`) selecting what's rendered. `app/model.rs` holds the small data types (`BrowserItem`, `SparqlResult`, `SearchResult`). There's no per-view sub-app or trait; adding a view means adding fields to `App`, a `View` variant, a render module, and match arms in `main.rs`.

Views with a text input (`Sparql`, `Search`) track an input-vs-results toggle (`sparql_mode_input` / `search_mode_input`); `Tab` switches focus and most result-navigation keys are guarded by `!..._mode_input` in the dispatch match.

### Rendering vs. input are fully separate

`ui/mod.rs` dispatches `View` to a render function per module (`ui/browser.rs`, `types.rs`, `sparql.rs`, `search.rs`, `bookmarks.rs`) plus a shared status bar (`ui/status.rs`) that shows contextual keybinding hints per view/mode. All keyboard input, for every view, is handled in one large `match (&app.view, key.code, key.modifiers)` in `main.rs::run` — there's no per-widget event handling; render modules are pure (`&App -> Frame`) and never mutate state.

### Display formatting is centralized and two-track

`rdf/display.rs`'s `DisplayContext` is the single place terms get turned into strings, and it deliberately has two families of methods that must not be conflated:
- `display_*` — human-readable (uses cached `rdfs:label`, prefix-shortened IRIs) for the UI.
- `sparql_*` — machine-readable, always full prefix:local or `<iri>` form (never a label), with proper literal escaping — used to build the "copy triple as SPARQL" text and to generate the queries in `rdf/sparql/mod.rs`.

Built-in prefixes and label caching both live here. When adding UI that shows terms, prefer extending `DisplayContext` rather than formatting terms elsewhere.

### Persistence

Only bookmarks are persisted, via `confy` (TOML, OS config dir) through `config.rs`'s `Config { bookmarks: Vec<String> }`. There's no other on-disk state.
