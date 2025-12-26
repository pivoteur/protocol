# dusk

Computes which pivots to close, then shows assets to pivot, aggregated by
blockchain.

![Assets to pivot by blockchain](../../quizzes/quiz05/a_assets/imgs/01-assets.png)

## Usage

`$ dusk <protocol> <date>`

where:
* `<protocol>` is the protocol-name, e.g. `PIVOT`
* `<date>` is the date to propose pivots, e.g. `2025-12-18`

## Revisions

* 1.07, 2025-12-26: if there are no pivots to close, report that explicitly
* 1.06, 2025-12-22: upgraded to match Composition-aware libraries
* 1.05, 2025-12-21: moved Asset and Measurable to their own modules
* 1.04, 2025-12-21: made git-fetch of pool names generic on path
* 1.03, 2025-12-18: sorting aggregated assets to pivot by USD-value
* 1.02, 2025-12-18: Generic sort-function for any Measurable-type
* 1.01, 2025-12-18: Sort pivots by USD-amount-pivoted
* 1.00, 2025-12-18: release
