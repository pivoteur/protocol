# `pools`

`$ pools <auth> <date>`

where:

* `<auth>` is the protocol that has pivot pools, e.g. `pivot`
* `<date>` is today's date, e.g. `$LE_DATE`

> n.b.: `PIVOT_DATA_DIR` must be set in the environment

## Sample run

`$ pools pivot $LE_DATE`

![Pivot pools for Pivot 
Protocol](../../quizzes/src/quiz10/b_pools/imgs/01-pools.png)

* [src](../../quizzes/src/quiz10/c_local_pools/mod.rs)

-----

# Revision History

* 2.02, 2026-05-05: brought into the automation framework
* 2.01, 2026-04-29: trustless version, no authentication needed
* 1.00, 2026-02-09: first version, reads pivot pools from github

