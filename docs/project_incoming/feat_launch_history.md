---
priority: P2
---

# Launch History

## Summary

Track flight statistics across launches so the player can review past performance and compare their best flights over time.

## Motivation

Currently each launch is fire-and-forget — once you reset, all data from the previous flight is gone. A launch history turns the sandbox into something with progression and replayability: "Can I beat my altitude record? My longest flight time?"

## Data to capture per flight

- **Timestamp** (wall-clock time of launch)
- **Max altitude** (m)
- **Max velocity** (m/s)
- **Flight duration** (launch to grounded)
- **Landing outcome** (soft landing / crash / still flying when reset)
- **Rocket config at launch** (dimensions, flight parameters — enables comparing across different builds)
- **Full trajectory** — position (and optionally velocity) sampled over time, for replay/visualization

## Retention

Keep the last **10 flights** in a ring buffer. Oldest flight is evicted when a new one is recorded.

## Display

In-world display: a **poster/board in the rocket shop** showing your flight record and achievements. This keeps the UI diegetic and fits the shop environment. Could be:
- A wall-mounted leaderboard mesh with dynamic text
- An egui overlay triggered by interacting with / looking at the poster
- Trajectory plots rendered onto the poster (altitude-over-time curves for recent flights)

Personal bests highlighted (max altitude, longest flight, etc.).

## Achievements / Challenges

Build a lightweight achievement system on top of the flight history:
- **Milestone achievements**: "Reach 100m", "Reach 500m", "Break the sound barrier"
- **Consistency challenges**: "Soft-land 3 times in a row", "10 flights without a crash"
- **Exploration**: "Longest hang time", "Highest velocity"
- Display unlocked achievements on the shop poster alongside the flight log

## Persistence

Skip for initial implementation — history lives only in the current session's memory (Bevy resource). Persistence (local file / localStorage for WASM) can be added later.

## Priority

Medium — good quality-of-life feature that adds replayability with relatively low complexity.
