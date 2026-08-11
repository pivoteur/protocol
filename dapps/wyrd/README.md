# `wyrd`

Calculates a close pivot.

`$ wyrd <protocol> <path> <ix> <tx_id> <amount>`

where:

* `<protocol>` is the protocol where the close pivot is, e.g.: PIVOT
* `<path>` is where the close pivots are, e.g.: data/pivots/close/raw
* `<ix>` is the close pivot call id, e.g.: 5
* `<tx_id>` is the close pivot swap transaction id, e.g.: some URL
* `<amount>` is the amount swapped to to close the pivot, e.g.: 0.17

* [source](../../quizzes/src/quiz08/f_wyrd_jr/mod.rs)

## Revisions

* 2.01, 2026-08-10: added opened-column to close pivot output
* 2.00, 2026-08-10: shifted to a call-to-close-pivot-type transformation
* 1.05, 2026-07-29: library-upgrage
* 1.04, 2026-07-10: add debugging information; correct pool-path algorithm
* 1.03, 2026-07-05: use clap to process arguments and for usage-documentation
* 1.02, 2026-05-18: made path an argument, not an environmental variable
* 1.01, 2026-05-17: made protocol- and path-agnostic.
* 1.00, 2026-05-05: release

