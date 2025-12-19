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

* 1.03, 2025-12-18: sorting aggregated assets to pivot by USD-value
* 1.02, 2025-12-18: Generic sort-function for any Measurable-type
* 1.01, 2025-12-18: Sort pivots by USD-amount-pivoted
* 1.00, 2025-12-18: release
