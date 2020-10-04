This library provides helpers to convert iterators of results into results
of collections of values or errors. The standard library provides `FromIterator`
for Result directly, which serves many of the same goals as this crate. However,
the standard library implementation only allows collecting into a result of all
values or the _first_ error. This crate differs in that all errors are preserved
in the chosen collection type, allowing more complete error reporting with
similar ergonomics.
