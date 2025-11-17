# libs

Collection of utilities for pivot-management.

* [fetchers](src/fetchers.rs): fetch data from REST endpoints
* [parsers](src/parsers.rs): parsing functions to convert raw data to RUST-types
* [tables](src/tables.rs): reify rows of data to tables
* types: types used throughout the library
  * [util](src/types/util.rs): the `Id` and `CsvHeader` types
  * [pivots](src/types/pivots.rs): structure and operations for the Pivot-type
