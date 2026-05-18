# `wyrd`

Calculates a close pivot.

`$ wyrd <protocol> <path> <ix> <tx_id> <amount>`
where
	* `<protocol>` is the protocol where the close pivot is, e.g.: PIVOT
	* `<path>` is where the close pivots are, e.g.: data/pivots/close/raw
	* `<ix>` is the close pivot call id, e.g.: 5
	* `<tx_id>` is the close pivot swap transaction id, e.g.: some URL
	* `<amount>` is the amount swapped to to close the pivot, e.g.: 0.17

* [source](../../quizzes/src/quiz08/b_wyrd/mod.rs)

## Revisions

* 1.02, 2026-05-18: made path an argument, not an environmental variable
* 1.01, 2026-05-17: made protocol- and path-agnostic.
* 1.00, 2026-05-05: release

