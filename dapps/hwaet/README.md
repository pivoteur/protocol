# `hwaet`

Computes the available assets for all pivot pools and reports on the health of
same.

`$ hwaet [--debug] <protocol> <date>`

where:

* `[-d|--debug]` shows `hwaet`-processing by pivot pool
* `<protocol>` is the pivot pool protocol, e.g.: `PIVOT`
* `<date>` is the date to evaluate the pivot pools, e.g.: `$LE_DATE`

[src](../../quizzes/src/quiz07/d_health/mod.rs)

-----

## Revision history

* 1.03, 2026-06-27: Clarified which pool is over-committed on error.
* 1.02, 2026-06-01: Moved functionality to processing-module in libs.
* 1.01, 2026-05-26: Added debugging-flow to pool-processing.
* 1.00, 2026-05-17: release

