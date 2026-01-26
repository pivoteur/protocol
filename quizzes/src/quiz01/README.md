# quiz01

## Read and parse a set of open pivots

* [tweet](https://x.com/pivocateur/status/1977865103178403984)
* [BTC+ETH open 
pivots](https://raw.githubusercontent.com/pivoteur/pivoteur.github.io/refs/heads/main/data/pivots/open/raw/btc-eth.tsv)
data file.

This project is more complicated that reading and parsing a file, as there
is a REST-endpoint involved and some data-structuring, so, let's break this
problem down into a set of stepping-stones to the solution.

* [Read](src/a_read) the open pivots from a REST endpoint
* [Parse](src/b_table) the pivots into a table
