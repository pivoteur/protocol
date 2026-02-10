# hound

![Evaluate all pivot pools](../../quizzes/quiz04/c_iterate/imgs/03-condensed.png)

Evaluates all pivot pools, recommending close pivots while listing pools
that have no close pivots.

[Source code](../../quizzes/quiz04/c_iterate/src/main.rs)

## Future work

* list pivot assets and amounts for close pivots

## Versions

* 2.01, 2025-12-17: incorporated `app_name()` into usage and error messaging.
* 2.00, 2025-12-17: renamed dapp from `hound` to `phound` as input-data now requires blockchain, so I'm doing a blue (`hound`) - green (`phound`) deployment until the next major overhaul calls for a revolutionary-release.
-----
* 1.05, 2025-12-17, incremental improvement: separated processing from 
reporting proposed pivot closes.
* [1.04](https://github.com/pivoteur/biz/tree/main/blog/2025/12/16), 2025-12-16:
 assets now reported by blockchain
* [1.03](https://github.com/pivoteur/biz/tree/main/blog/2025/12/11), 2025-12-11:
reports condense to one close pivot (for potentially multiple open 
pivots) per row, all metadata now included in the row

