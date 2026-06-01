# lau-challenge

> Challenge/arena system for Lau — kids create challenges and the system validates solutions

## What This Does

Challenge/arena system for Lau — kids create challenges and the system validates solutions. Part of the PLATO/LAU ecosystem — a mathematically rigorous framework for building educational agents that learn, teach, and evolve.

## The Key Idea

This crate implements the core abstractions needed for its domain, with a focus on correctness, composability, and conservation guarantees. Every public type is serializable (serde), every algorithm is tested, and every invariant is verified.

## Install

```bash
cargo add lau-challenge
```

## Quick Start

See the API Reference below for complete usage. Key entry points:

```rust
use lau_challenge::*;
// See types and methods below for complete usage
```

## API Reference

```rust
pub enum ChallengeCategory 
pub enum Constraint 
pub enum ScoringRule 
pub struct Challenge 
    pub fn validate_difficulty(difficulty: u32) -> bool 
    pub fn check_constraints(&self, submission: &Submission) -> Option<&Constraint> 
pub struct Submission 
pub struct ChallengeResult 
pub struct ChallengeArena 
    pub fn new() -> Self 
    pub fn create_challenge(&mut self, challenge: Challenge) -> String 
    pub fn submit(&mut self, submission: Submission) -> ChallengeResult 
    pub fn leaderboard(&self, challenge_id: &str) -> Vec<&ChallengeResult> 
    pub fn player_submissions(&self, player: &str) -> Vec<&Submission> 
    pub fn popular_challenges(&self) -> Vec<&Challenge> 
    pub fn challenges_by_category(&self, cat: &ChallengeCategory) -> Vec<&Challenge> 
pub fn prebuilt_challenges() -> Vec<Challenge> 
    pub fn simple(challenge_id: &str, player: &str, tick: u64, score: f64) -> Self 
```

## How It Works

Read the source in `src/` for full implementation details. All algorithms are documented with inline comments explaining the mathematical foundations.

## The Math

This crate implements formal mathematical constructs. See the source documentation for theorem statements and proofs of correctness.

## Testing

**28 tests** covering construction, serialization, correctness properties, edge cases, and composability with other lau-* crates.

## License

MIT
