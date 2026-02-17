# Automation Workflow

The daily workflow for Pivot protocol can be viewed as phasal.

```mermaid
---
title: Protocol Pivot Arbitrage Workflow
---
stateDiagram-v2
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
   class pools inUse

   class Tests wip
   class Setup wip
   class Closes wip
   class WorkFlow wip

   class report manual
   class ReportwithoutUpdatingDatabase manual
   class UpdateDb manual
   class UpdateDbwithoutReporting manual
   class UpdateDbwithoutReporting1 manual

   [*] --> Workflow
   Workflow --> Finalize
   Finalize --> [*]

   Workflow: The day-to-day process of opening and closing pivots
   state Workflow {
      direction LR

      [*] --> Tests
      Tests --> Setup
      Setup --> Closes
      Closes --> Opens
      Opens --> [*]
   }

   Finalize: Update Database and Report results
   state Finalize {
      [*] --> UpdateDb
      UpdateDb --> report 
      report --> [*]

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
            direction LR

            [*] --> Run
            Run --> [*]

            Run: cargo run
         }
         Rep: Automation Status Report
         state Rep {
            direction LR

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
            direction LR

         [*] --> bae
         bae --> UpdateDbwithoutReporting
         UpdateDbwithoutReporting --> [*]

         UpdateDbwithoutReporting: Update database
      }

      Pools: Scan active pivot pools
      state Pools {
            direction LR

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
            direction LR

            [*] --> virtsz
            virtsz --> [*]
         }

         AdjustVirtsz: Update virtual pivots
      }
   }

   Closes: Close Pivots
   state Closes {
      [*] --> ScanCloses
      ScanCloses --> CloseAndDistribute
      CloseAndDistribute --> [*]

      ScanCloses: Scan Pools for Close calls
      state ScanCloses {
         direction LR

         [*] --> dusk
         dusk --> [*]
      }

      CloseAndDistribute: Close Pivots and Distribute Gains
      state CloseAndDistribute {
         [*] --> Close
         Close --> Distribute
         Distribute --> [*]

         Close: Close Pivots

         Distribute: Distribute or Reinvest Gains
         state Distribute {
            direction LR

            [*] --> distribute
            distribute --> reinvest
            reinvest --> [*]
         }
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

