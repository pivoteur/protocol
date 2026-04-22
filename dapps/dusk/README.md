# dusk

Computes which pivots to close, then shows assets to pivot, aggregated by
blockchain.

![Assets to pivot by 
blockchain](../../quizzes/src/quiz05/b_dusk_min/imgs/01-dusk.png)

[Source](../../quizzes/src/quiz05/b_dusk_min/mod.rs)

## Usage

`$ dusk [--min] <protocol> <date>`

where:
* `[--min]` is to show only the close pivot calls and no meta-data
* `<protocol>` is the protocol-name, e.g. `PIVOT`
* `<date>` is the date to propose pivots, e.g. `2025-12-18`

## Revisions

* 2.02, 2026-04-22: `dusk` now determines pivot pools from `libs/pool-assets.js`
* 2.01, 2026-04-10: moved `dusk` into to the furnctional and unit test framework
* 2.00, 2026-04-07: Permit `--min` to show only close pivot calls
* 1.10, 2026-03-14: Brought into the functional-testing framework
* 1.09, 2026-01-25: close pivot calls are now indexed
* 1.08, 2025-12-28: Token-aliases handle synthetics and wrapped tokens;
compacted no-close-calls pivot-pools sections.
* 1.07, 2025-12-26: if there are no pivots to close, report that explicitly
* 1.06, 2025-12-22: upgraded to match Composition-aware libraries
* 1.05, 2025-12-21: moved Asset and Measurable to their own modules
* 1.04, 2025-12-21: made git-fetch of pool names generic on path
* 1.03, 2025-12-18: sorting aggregated assets to pivot by USD-value
* 1.02, 2025-12-18: Generic sort-function for any Measurable-type
* 1.01, 2025-12-18: Sort pivots by USD-amount-pivoted
* 1.00, 2025-12-18: release
