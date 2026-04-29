# `pools`

`$ pools <auth> <date>`

where:

* `<auth>` is the protocol that has pivot pools, e.g. `pivot`
* `<date>` is today's date, e.g. `$LE_DATE`

> n.b.: `PIVOT_DATA_DIR` must be set in the environment

## Sample run

`$ pools pivot $LE_DATE`

```Javascript
const poolAssets = {
   generated: '2026-04-29',
   assets: [
      [ 'AVAX', 'UNDEAD' ],
      [ 'BTC', 'UNDEAD' ],
      [ 'UNDEAD', 'USDC' ],
      [ 'ETH', 'UNDEAD' ],
      [ 'BTC', 'AVAX' ],
      [ 'BTC', 'ETH' ],
      [ 'BTC', 'USDC' ]
   ]
};
```

* [src](mod.rs)

-----

# Revision History

* 2.01, 2026-04-29: trustless version, no authentication needed
* 1.00, 2026-02-09: first version, reads pivot pools from github

