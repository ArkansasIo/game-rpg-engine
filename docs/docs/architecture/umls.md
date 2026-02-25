---
title: "UMLs"
sidebar_position: 1
---

# NexusStudio UMLs

This page documents core system relationships and runtime flows.

## Class Diagram (Shared 2D RPG Core)

```mermaid
classDiagram
    class Grid2D~T~ {
      +width() usize
      +height() usize
      +get(x,y) Option~T~
      +set(x,y,value) bool
      +neighbors4(x,y) Vec~Coord2~
    }

    class Grid2DTime~T~ {
      +get(x,y,t) Option~T~
      +set(x,y,t,value) bool
      +get_time_slice(t) Grid2D~T~
      +set_time_slice(t,slice) bool
    }

    class Grid2DHistory~T~ {
      +begin_step() bool
      +set(x,y,value) bool
      +commit_step() bool
      +undo() bool
      +redo() bool
    }

    class World2D {
      +try_move(from,dir) (Coord2,Vec~WorldEvent~)
      +interact(at) Vec~WorldEvent~
      +should_start_encounter(at,roll) bool
    }

    class EncounterTable {
      +pick_for_terrain(terrain,roll) EncounterPack
    }

    class BattleState {
      +apply_party_action(action) Vec~BattleEvent~
      +apply_enemy_action(action) Vec~BattleEvent~
      +end_round_tick()
    }

    class Party {
      +members Vec~Fighter~
      +inventory HashMap~String,u32~
    }

    class Fighter {
      +name String
      +stats Stats
      +statuses Vec~StatusEffect~
    }

    class Stats {
      +hp i32
      +mp i32
      +str_ i32
      +agi i32
      +vit i32
      +int_ i32
      +luk i32
    }

    Grid2DTime --> Grid2D
    Grid2DHistory --> Grid2D
    World2D --> Grid2D
    World2D --> EncounterTable
    BattleState --> Party
    BattleState --> Fighter
    Fighter --> Stats
```

## Sequence Diagram (Movement -> Encounter -> Battle)

```mermaid
sequenceDiagram
    participant Player
    participant World as World2D
    participant Enc as EncounterTable
    participant Battle as BattleState

    Player->>World: try_move(from, dir)
    World-->>Player: moved + world events
    Player->>World: should_start_encounter(tile, roll)
    alt encounter triggers
      World->>Enc: pick_for_terrain(terrain, roll)
      Enc-->>World: EncounterPack
      World-->>Player: start battle
      Player->>Battle: apply_party_action(Attack/Spell/Item/Run)
      Battle-->>Player: BattleEvents
      Battle->>Battle: end_round_tick()
    else no encounter
      World-->>Player: continue exploration
    end
```

## State Diagram (Battle)

```mermaid
stateDiagram-v2
    [*] --> Ongoing
    Ongoing --> Ongoing: action resolved / round tick
    Ongoing --> Escaped: successful Run
    Ongoing --> PartyWon: all enemies defeated
    Ongoing --> EnemyWon: all party members defeated
    Escaped --> [*]
    PartyWon --> [*]
    EnemyWon --> [*]
```

## Notes

- `Grid2DHistory` stores deltas, not full snapshots.
- `World2D` drives exploration and trigger execution.
- `BattleState` resolves turn actions and status effects.
- `EncounterTable` maps terrain to weighted monster packs.

