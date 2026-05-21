# `reinvested`

`$ reinvested <token_a> <token_b> <pivot_count> <amount> <url>`

where:

* `<token_a>`     is the reinvested token, left side of pool. e.g. `AVAX`
* `<token_b>`     is the paired token, right side of pool. e.g. `BTC`
* `<pivot_count>` is the number of pivots closed. e.g. `2`
* `<amount>`      is the amount reinvested. e.g. `0.59`
* `<url>`         is the twitter URL. e.g. `x.com/pivocateur`

Sends a telegram regarding the amount reinvested into pivot pools.

> n.b: "REINVESTED_BOT" must be set as an environmental variable locally

* [source](../../quizzes/src/quiz11/a_reinvested/mod.rs)

## Revisions

* 1.00, 2026-05-18: release, and adding into the quizzes testing framework

