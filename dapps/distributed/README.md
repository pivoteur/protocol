# `distributed`

`$ distributed <token_a> <token_b> <amount> <tweet_url> <tx_url>`

where:

* `<token_a>`     is the distributed token, left side of pool. e.g. `AVAX`
* `<token_b>`     is the paired token, right side of pool. e.g. `BTC`
* `<amount>`      is the amount distributed. e.g. `0.59`
* `<tweet_url>`   is the twitter URL. e.g. `x.com/pivocateur`
* `<tx_url>`      is that snowtrace url. e.g. `snowtrace.io`

Creates a distribution message for investors.

> n.b: "REINVESTED_BOT" must be set as an environmental variable locally

* [source](../../quizzes/src/quiz11/a_reinvest/mod.rs)

## Revisions

* 1.00, 2026-05-19: release, and adding into the quizzes testing framework

