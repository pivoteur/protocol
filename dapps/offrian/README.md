# `offrian`

Some pivots need to be broken up into smaller trades. Are these smaller trades
viable? This determination is `offrian`'s job.

## Usage

![`offrian` usage](../../quizzes/src/quiz08/e_offrian/imgs/01a-usage.png)

```
offrian, version 1.00

Usage:

$ offrian [--debug] <protocol> <ix> <part>

where:

* [-d|--debug] show debug information
* <protocol> is the protocol to make the counter-offer, e.g.: PIVOT
* <ix> is the call being countered, e.g. 1
* <part> is the subset of the call being countered,
  e.g.: 3 is a 1/3rd counter-offer
```

### e.g.

> `$ offrian -d pivot 1 36`

depending on the call will get the following output:

![`offrian` output](../../quizzes/src/quiz08/e_offrian/imgs/01b-runoff.png)

* [src](../../quizzes/src/quiz08/e_offrian/mod.rs)

------

## Revisions

* 1.00, 2026-06-19: release
* 0.90, 2026-06-16: pre-release, reading calls

