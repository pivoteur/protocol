# quiz07

We're reading the pivot pools' assets, and we already did the open-pivot
calculations. By combining these two efforts we see how much we have left in
a pivot pool. With this information, we can determine which pools to open
pivots on.

We can determine which direction to open the pivot on the pool using
the EMA20 and the δ, but that work is for a later quiz.

* [a_ssets](a_ssets): fetching quotes, assets, open pivots
* [b_virtual](b_virtual): computes assets committed to virtual swaps
* [c_avail](c_avail): Collates assets, virtual pivots, and computes available 
assets

