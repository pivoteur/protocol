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

* 1.06, 2026-07-14: `pools = [` becomes `pools: [` to conform to Javascript 
syntax
* 1.05, 2026-07-10, bug-fix: added commas after each pool-object to fix syntax
* 1.04, 2026-07-08, hot-fix: handle pivot pools with one-sided assets
* 1.03, 2026-07-05: using clap to process arguments and for usage-documentation
* 1.02, 2026-06-27: Clarified which pool is over-committed on error.
* 1.01, 2026-05-26: Added debugging-flow to pool-processing.
* 1.00, 2026-05-17: release

