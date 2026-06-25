# `offrian`

Some pivots need to be broken up into smaller trades. Are these smaller trades
viable? This determination is `offrian`'s job.

## Usage

![`offrian` usage](../../quizzes/src/quiz08/e_offrian/imgs/01a-usage.png)

```
offrian, version 1.01

Usage:

$ offrian [--debug] <protocol> <ix> <target>

where:

* [-d|--debug] show debug information
* <protocol> is the protocol to make the counter-offer, e.g.: PIVOT
* <ix> is the call being countered, e.g. 1
* <target> target volume for the new pivot (e.g. 1000)
```

### e.g.

> `$ offrian -d pivot 1 36`

depending on the call will get the following output:

### Rejected offer

![`offrian` output](../../quizzes/src/quiz08/e_offrian/imgs/01b-rejected.png)

### Accepted offer

![`offrian` output](../../quizzes/src/quiz08/e_offrian/imgs/01c-runoff.png)

* [src](../../quizzes/src/quiz08/e_offrian/mod.rs)

------

## Revisions

* 1.03, 2026-06-25: Move virtual pivot functions to libs::processors:virtuals
* 1.02, 2026-06-24: changed partition to target volume
* 1.01, 2026-06-19 (again): added tests and corrected formulæ
* 1.00, 2026-06-19: release
* 0.90, 2026-06-16: pre-release, reading calls

