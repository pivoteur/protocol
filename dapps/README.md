# Automation Workflow

The daily workflow for Pivot protocol can be viewed as phasal.

```mermaid
stateDiagram-v2

   accTitle: Protocol Pivot Arbitrage Workflow
   accDescription: The day-to-day process of opening and closing pivots.

   classDef inUse fill:lime,color:black
   classDef wip fill:yellow,color:black
   classDef manual fill:violet,color:white
   classDef notYet fill:black,color:white

   class Tarp inUse
   class Run inUse
   class itr inUse
   class bae inUse
   class dusk inUse
   class virtsz inUse

   class Tests wip
   class Setup wip
   class Closes wip

   class report manual
   class ReportwithoutUpdatingDatabase manual
   class UpdateDb manual
   class UpdateDbwithoutReporting manual
   class UpdateDbwithoutReporting1 manual
   class Finalize manual

   direction LR
   [*] --> Tests
   Tests --> Setup
   Setup --> Finalize
   Setup --> Closes
   Closes --> Finalize
   Closes --> Opens
   Opens --> Finalize
   Finalize --> [*]

   Finalize: Update Database and Report results
   state Finalize {
      [*] --> report
      report --> UpdateDb
      UpdateDb --> [*]

      UpdateDb: update database
   }

   Tests: Health Check
   state Tests {
      [*] --> Integration
      Integration --> [*]

      Integration: Tests
      state Integration {
         Tarp: cargo tarpaulin
         [*] --> itr
         itr --> Tarp
         Tarp --> Func
         Func --> Rep
         Rep --> [*]
         Func: Runs my functional test framework
         state Func {
            [*] --> Run
            Run --> [*]

            Run: cargo run
         }
         Rep: Automation Status Report
         state Rep {
            [*] --> ReportwithoutUpdatingDatabase
            ReportwithoutUpdatingDatabase --> [*]

            ReportwithoutUpdatingDatabase: report
         }
      }
   }

   Setup: Setup
   state Setup {
      [*] --> Quotes
      Quotes --> Pools
      Pools --> AdjustVirtualPivots
      AdjustVirtualPivots --> [*]

      Quotes: Ingest quotes
      state Quotes {
         [*] --> bae
         bae --> UpdateDbwithoutReporting
         UpdateDbwithoutReporting --> [*]

         UpdateDbwithoutReporting: Update database
      }

      Pools: Scan active pivot pools
      state Pools {
         [*] --> pools
         pools --> UpdateDbwithoutReporting1
         UpdateDbwithoutReporting1 --> [*]

         UpdateDbwithoutReporting1: Update database
      }

      AdjustVirtualPivots: Adjust Virtual Open Pivots
      state AdjustVirtualPivots {
         [*] --> ScanVirtsz
         ScanVirtsz --> AdjustVirtsz
         AdjustVirtsz --> [*]

         ScanVirtsz: Scan Pivot Pools for Virtual Pivots
         state ScanVirtsz {
            [*] --> virtsz
            virtsz --> [*]
         }

         AdjustVirtsz: Update virtual pivots
      }
   }

   Closes: Close Pivots
   state Closes {
      [*] --> ScaClosesn
      ScanCloses --> Close
      Close --> UpdateDb
      UpdateDb --> report
      report --> Distribute
      Distribute --> [*]

      ScanCloses: Scan Pools for Close calls
      state ScanCloses {
         [*] --> dusk
         dusk --> [*]
      }
   }

   Opens: Open New Pivots
   state Opens {
      [*] --> ScanOpens
      ScanOpens --> Call
      Call --> Open
      Open --> OpenOrHedge
      OpenOrHedge --> [*]

      ScanOpens: Scan Pools for (virtual and real) available assets
      Call: Analyze EMA20 Trendlines to make open pivot call
      Open: Open a new pivot
      OpenOrHedge: Open a hedge against the pivot or open the dual pivot
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

