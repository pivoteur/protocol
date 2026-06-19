# `offrian`

Recomputes a call to a fractional amount. This is useful when slippage is so
high that closing a pivot is not profitable, but a partial close works.

## Usage

![`offrian` usage](imgs/01a-usage.png)

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

![`offrian` output](imgs/01b-runoff.png)

* [src](mod.rs)

