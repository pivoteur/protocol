# quiz06

We have automation in place for close-pivots, but we have no automation around 
the just-as-important open-pivots workflow.

Let's get to work!

We've solved reading open-pivot tables. Let's read the pivot pool tables and 
combine those states to determine allocatable assets.

* [a_pool_table](a_pool_table): scans a (pivot) pool table

-----

## A new line of inquiry

This first step (--^) lead to an interesting line of inquiry:

* What pivot pools have enough assets to pivot? (of course), but also:
* What pivot pools are underfunded? and, if so:

* should they be canned? consolidated? what?

Let's check pivot pool funding.

To check every pool's funding, we must first know what every pool is. We've
already done this for open pivots, so we can appropriate that work for this
effort, as well.

* [b_pools](b_pools): read the pool-names from git


