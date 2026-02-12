# Automation Workflow

The daily workflow for Pivot protocol can be viewed as phasal.

```mermaid
stateDiagram-v2
   direction LR
   [*] --> Tests
   Tests --> Quotes
   Quotes --> Closes
   Closes --> [*]

   Tests: Health Check
   state Tests {
      [*] --> Integration
      Integration --> [*]

      Integration: Integration Tests
      state Integration {
         [*] --> itr
         itr --> [*]
      }
   }

   Quotes: Ingest quotes
   state Quotes {
      [*] --> gecko
      gecko --> [*]
   }

   Closes: Close Pivots
   state Closes {
      [*] --> Scan
      Scan --> [*]

      Scan: Scan Pools for Close calls
      state Scan {
         [*] --> dusk
         dusk --> [*]
      }
   }
```

# dapps

Applications for protocol workflow

## Integration Testing

### `itr`: Integration tester

Iterates `cargo build` over each subdir in `<dir>`

* [itr](itr)

## Released

### `dusk`: close pivots

* [dusk](dusk): aggregates assets to pivot by blockchain

Standalone dapps that also support `dawn` include:

### `assets`: state of all pools

* [assets](assets): partitions pools by TVL

### `virtsz`: Assets committed to virtual pivots

* [virtsz](virtsz)

## WIP / Works in Progress

## `dawn`: open pivots

... very much a WIP

* [dawn](dawn): the start of a start, reading pivot pool assets

## Archived

### Evolution of `dusk`

* [chihuahua](chihuahua): close recommendations on one pivot pool
* [basset](basset): close-pivot recommendations condensed to one trade per asset
* [phound / hound](hound): close pivots for all pivot pools

