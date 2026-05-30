use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Core types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ChallengeCategory {
    Building,
    Farming,
    Conservation,
    AgentTraining,
    Speedrun,
    Creativity,
    Collaboration,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Constraint {
    MaxBlocks(usize),
    MaxTime(u64),
    MinConservationScore(f64),
    RequiredMaterial(String),
    Budget(f64),
    NoAgentHelp,
    MustUseBiome(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScoringRule {
    HighestScore,
    FastestTime,
    BestConservation,
    MostCreative,
    HighestYield,
    FewestResources,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Challenge {
    pub id: String,
    pub title: String,
    pub description: String,
    pub creator: String,
    pub difficulty: u32,
    pub category: ChallengeCategory,
    pub constraints: Vec<Constraint>,
    pub scoring: ScoringRule,
    pub time_limit: Option<u64>,
}

impl Challenge {
    /// Validate that a difficulty value is in range 1-5.
    pub fn validate_difficulty(difficulty: u32) -> bool {
        (1..=5).contains(&difficulty)
    }

    /// Check every constraint against a submission, returning the first violation or `None`.
    pub fn check_constraints(&self, submission: &Submission) -> Option<&Constraint> {
        self.constraints.iter().find(|c| match c {
            Constraint::MaxBlocks(max) => {
                submission.blocks_used.is_some_and(|b| b > *max)
            }
            Constraint::MaxTime(max) => submission.tick > *max,
            Constraint::MinConservationScore(min) => submission.conservation_score < *min,
            Constraint::RequiredMaterial(mat) => !submission.materials_used.contains(mat),
            Constraint::Budget(max) => submission.cost > *max,
            Constraint::NoAgentHelp => submission.agent_help_used,
            Constraint::MustUseBiome(biome) => {
                submission.biome_used.as_ref() != Some(biome)
            }
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Submission {
    pub challenge_id: String,
    pub player: String,
    pub tick: u64,
    pub score: f64,
    pub solution_data: String,
    // --- fields used for constraint checking ---
    pub blocks_used: Option<usize>,
    pub conservation_score: f64,
    pub materials_used: Vec<String>,
    pub cost: f64,
    pub agent_help_used: bool,
    pub biome_used: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeResult {
    pub submission: Submission,
    pub rank: usize,
    pub verified: bool,
    pub conservation_error: f64,
}

// ---------------------------------------------------------------------------
// Arena
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChallengeArena {
    challenges: HashMap<String, Challenge>,
    submissions: HashMap<String, Vec<Submission>>,
    results: HashMap<String, Vec<ChallengeResult>>,
}

impl ChallengeArena {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new challenge and return its id.
    pub fn create_challenge(&mut self, challenge: Challenge) -> String {
        let id = challenge.id.clone();
        self.challenges.insert(id.clone(), challenge);
        self.submissions.insert(id.clone(), Vec::new());
        self.results.insert(id.clone(), Vec::new());
        id
    }

    /// Submit a solution, validate constraints, score, and return a result.
    pub fn submit(&mut self, submission: Submission) -> ChallengeResult {
        let cid = submission.challenge_id.clone();

        let (verified, conservation_error) = match self.challenges.get(&cid) {
            Some(ch) => {
                let violated = ch.check_constraints(&submission);
                let ce = (1.0 - submission.conservation_score).abs();
                (violated.is_none(), ce)
            }
            None => (false, 0.0),
        };

        let result = ChallengeResult {
            submission: submission.clone(),
            rank: 0,
            verified,
            conservation_error,
        };

        self.submissions
            .entry(cid.clone())
            .or_default()
            .push(submission);

        self.results
            .entry(cid.clone())
            .or_default()
            .push(result);

        self.recalculate_ranks(&cid);

        self.results
            .get(&cid)
            .and_then(|v| v.last())
            .cloned()
            .unwrap()
    }

    fn recalculate_ranks(&mut self, challenge_id: &str) {
        let challenge = self.challenges.get(challenge_id);
        let lower_is_better = challenge.is_some_and(|c| {
            matches!(
                c.scoring,
                ScoringRule::FastestTime
                    | ScoringRule::FewestResources
                    | ScoringRule::BestConservation
            )
        });

        if let Some(results) = self.results.get_mut(challenge_id) {
            results.sort_by(|a, b| {
                if lower_is_better {
                    a.submission.score.partial_cmp(&b.submission.score).unwrap()
                } else {
                    b.submission.score.partial_cmp(&a.submission.score).unwrap()
                }
            });
            for (i, r) in results.iter_mut().enumerate() {
                r.rank = i + 1;
            }
        }
    }

    /// Return leaderboard for a challenge, sorted by rank.
    pub fn leaderboard(&self, challenge_id: &str) -> Vec<&ChallengeResult> {
        let mut v: Vec<_> = self
            .results
            .get(challenge_id)
            .map(|r| r.iter().collect())
            .unwrap_or_default();
        v.sort_by_key(|r| r.rank);
        v
    }

    /// All submissions by a given player.
    pub fn player_submissions(&self, player: &str) -> Vec<&Submission> {
        self.submissions
            .values()
            .flat_map(|v| v.iter())
            .filter(|s| s.player == player)
            .collect()
    }

    /// Challenges sorted by submission count (most popular first).
    pub fn popular_challenges(&self) -> Vec<&Challenge> {
        let mut v: Vec<_> = self.challenges.values().collect();
        v.sort_by(|a, b| {
            let ca = self.submissions.get(&a.id).map_or(0, |v| v.len());
            let cb = self.submissions.get(&b.id).map_or(0, |v| v.len());
            cb.cmp(&ca)
        });
        v
    }

    /// Challenges filtered by category.
    pub fn challenges_by_category(&self, cat: &ChallengeCategory) -> Vec<&Challenge> {
        self.challenges
            .values()
            .filter(|c| &c.category == cat)
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Pre-built challenges
// ---------------------------------------------------------------------------

pub fn prebuilt_challenges() -> Vec<Challenge> {
    vec![
        Challenge {
            id: "tower-of-balance".into(),
            title: "Tower of Balance".into(),
            description: "Build the tallest tower using at most 50 blocks while keeping a conservation score above 0.95.".into(),
            creator: "system".into(),
            difficulty: 3,
            category: ChallengeCategory::Building,
            constraints: vec![
                Constraint::MaxBlocks(50),
                Constraint::MinConservationScore(0.95),
            ],
            scoring: ScoringRule::HighestScore,
            time_limit: None,
        },
        Challenge {
            id: "speed-farm".into(),
            title: "Speed Farm".into(),
            description: "Harvest 100 crops within 500 ticks.".into(),
            creator: "system".into(),
            difficulty: 2,
            category: ChallengeCategory::Farming,
            constraints: vec![
                Constraint::MaxTime(500),
            ],
            scoring: ScoringRule::HighestYield,
            time_limit: Some(500),
        },
        Challenge {
            id: "conservation-master".into(),
            title: "Conservation Master".into(),
            description: "Maintain a conservation error below 0.01 for 100 ticks.".into(),
            creator: "system".into(),
            difficulty: 4,
            category: ChallengeCategory::Conservation,
            constraints: vec![
                Constraint::MinConservationScore(0.99),
                Constraint::MaxTime(100),
            ],
            scoring: ScoringRule::BestConservation,
            time_limit: Some(100),
        },
        Challenge {
            id: "solo-agent".into(),
            title: "Solo Agent".into(),
            description: "Train an agent to 0.9 accuracy without any external help.".into(),
            creator: "system".into(),
            difficulty: 5,
            category: ChallengeCategory::AgentTraining,
            constraints: vec![
                Constraint::NoAgentHelp,
            ],
            scoring: ScoringRule::HighestScore,
            time_limit: None,
        },
    ]
}

// ---------------------------------------------------------------------------
// Helper for building test submissions
// ---------------------------------------------------------------------------

impl Submission {
    pub fn simple(challenge_id: &str, player: &str, tick: u64, score: f64) -> Self {
        Self {
            challenge_id: challenge_id.into(),
            player: player.into(),
            tick,
            score,
            solution_data: String::new(),
            blocks_used: None,
            conservation_score: 1.0,
            materials_used: Vec::new(),
            cost: 0.0,
            agent_help_used: false,
            biome_used: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_challenge(id: &str, cat: ChallengeCategory) -> Challenge {
        Challenge {
            id: id.into(),
            title: format!("Test {id}"),
            description: "A test challenge".into(),
            creator: "tester".into(),
            difficulty: 3,
            category: cat,
            constraints: vec![],
            scoring: ScoringRule::HighestScore,
            time_limit: None,
        }
    }

    #[test]
    fn difficulty_in_range() {
        assert!(Challenge::validate_difficulty(1));
        assert!(Challenge::validate_difficulty(5));
        assert!(!Challenge::validate_difficulty(0));
        assert!(!Challenge::validate_difficulty(6));
    }

    #[test]
    fn create_challenge_returns_id() {
        let mut arena = ChallengeArena::new();
        let c = make_challenge("c1", ChallengeCategory::Building);
        let id = arena.create_challenge(c);
        assert_eq!(id, "c1");
        assert!(arena.challenges.contains_key("c1"));
    }

    #[test]
    fn submit_returns_result() {
        let mut arena = ChallengeArena::new();
        arena.create_challenge(make_challenge("c1", ChallengeCategory::Building));
        let sub = Submission::simple("c1", "alice", 10, 42.0);
        let res = arena.submit(sub);
        assert!(res.verified);
        assert_eq!(res.submission.player, "alice");
    }

    #[test]
    fn leaderboard_sorted_by_rank() {
        let mut arena = ChallengeArena::new();
        arena.create_challenge(make_challenge("c1", ChallengeCategory::Building));
        arena.submit(Submission::simple("c1", "alice", 10, 30.0));
        arena.submit(Submission::simple("c1", "bob", 10, 50.0));
        let lb = arena.leaderboard("c1");
        assert_eq!(lb.len(), 2);
        assert_eq!(lb[0].rank, 1);
    }

    #[test]
    fn player_submissions_across_challenges() {
        let mut arena = ChallengeArena::new();
        arena.create_challenge(make_challenge("c1", ChallengeCategory::Building));
        arena.create_challenge(make_challenge("c2", ChallengeCategory::Farming));
        arena.submit(Submission::simple("c1", "alice", 10, 10.0));
        arena.submit(Submission::simple("c2", "alice", 20, 20.0));
        let subs = arena.player_submissions("alice");
        assert_eq!(subs.len(), 2);
    }

    #[test]
    fn challenges_by_category_filters() {
        let mut arena = ChallengeArena::new();
        arena.create_challenge(make_challenge("c1", ChallengeCategory::Building));
        arena.create_challenge(make_challenge("c2", ChallengeCategory::Farming));
        let building = arena.challenges_by_category(&ChallengeCategory::Building);
        assert_eq!(building.len(), 1);
        assert_eq!(building[0].id, "c1");
    }

    #[test]
    fn popular_challenges_sorted_by_submissions() {
        let mut arena = ChallengeArena::new();
        arena.create_challenge(make_challenge("c1", ChallengeCategory::Building));
        arena.create_challenge(make_challenge("c2", ChallengeCategory::Farming));
        arena.submit(Submission::simple("c1", "a", 1, 1.0));
        arena.submit(Submission::simple("c1", "b", 1, 2.0));
        arena.submit(Submission::simple("c2", "a", 1, 3.0));
        let pop = arena.popular_challenges();
        assert_eq!(pop[0].id, "c1"); // 2 submissions
        assert_eq!(pop[1].id, "c2"); // 1 submission
    }

    // --- Constraint tests ---

    #[test]
    fn constraint_max_blocks_pass() {
        let c = make_challenge("c1", ChallengeCategory::Building);
        let mut sub = Submission::simple("c1", "a", 1, 1.0);
        sub.blocks_used = Some(10);
        assert!(c.check_constraints(&sub).is_none());
    }

    #[test]
    fn constraint_max_blocks_fail() {
        let mut c = make_challenge("c1", ChallengeCategory::Building);
        c.constraints.push(Constraint::MaxBlocks(5));
        let mut sub = Submission::simple("c1", "a", 1, 1.0);
        sub.blocks_used = Some(10);
        assert!(c.check_constraints(&sub).is_some());
    }

    #[test]
    fn constraint_max_time_fail() {
        let mut c = make_challenge("c1", ChallengeCategory::Speedrun);
        c.constraints.push(Constraint::MaxTime(100));
        let sub = Submission::simple("c1", "a", 200, 1.0);
        assert!(c.check_constraints(&sub).is_some());
    }

    #[test]
    fn constraint_min_conservation_fail() {
        let mut c = make_challenge("c1", ChallengeCategory::Conservation);
        c.constraints.push(Constraint::MinConservationScore(0.95));
        let mut sub = Submission::simple("c1", "a", 1, 1.0);
        sub.conservation_score = 0.8;
        assert!(c.check_constraints(&sub).is_some());
    }

    #[test]
    fn constraint_required_material_pass() {
        let mut c = make_challenge("c1", ChallengeCategory::Building);
        c.constraints.push(Constraint::RequiredMaterial("stone".into()));
        let mut sub = Submission::simple("c1", "a", 1, 1.0);
        sub.materials_used.push("stone".into());
        assert!(c.check_constraints(&sub).is_none());
    }

    #[test]
    fn constraint_budget_fail() {
        let mut c = make_challenge("c1", ChallengeCategory::Building);
        c.constraints.push(Constraint::Budget(100.0));
        let mut sub = Submission::simple("c1", "a", 1, 1.0);
        sub.cost = 150.0;
        assert!(c.check_constraints(&sub).is_some());
    }

    #[test]
    fn constraint_no_agent_help_fail() {
        let mut c = make_challenge("c1", ChallengeCategory::AgentTraining);
        c.constraints.push(Constraint::NoAgentHelp);
        let mut sub = Submission::simple("c1", "a", 1, 1.0);
        sub.agent_help_used = true;
        assert!(c.check_constraints(&sub).is_some());
    }

    #[test]
    fn constraint_must_use_biome_pass() {
        let mut c = make_challenge("c1", ChallengeCategory::Farming);
        c.constraints.push(Constraint::MustUseBiome("desert".into()));
        let mut sub = Submission::simple("c1", "a", 1, 1.0);
        sub.biome_used = Some("desert".into());
        assert!(c.check_constraints(&sub).is_none());
    }

    #[test]
    fn constraint_must_use_biome_fail() {
        let mut c = make_challenge("c1", ChallengeCategory::Farming);
        c.constraints.push(Constraint::MustUseBiome("desert".into()));
        let mut sub = Submission::simple("c1", "a", 1, 1.0);
        sub.biome_used = Some("forest".into());
        assert!(c.check_constraints(&sub).is_some());
    }

    // --- Prebuilt challenges ---

    #[test]
    fn prebuilt_has_four_challenges() {
        let p = prebuilt_challenges();
        assert_eq!(p.len(), 4);
    }

    #[test]
    fn prebuilt_ids() {
        let p = prebuilt_challenges();
        let ids: Vec<&str> = p.iter().map(|c| c.id.as_str()).collect();
        assert!(ids.contains(&"tower-of-balance"));
        assert!(ids.contains(&"speed-farm"));
        assert!(ids.contains(&"conservation-master"));
        assert!(ids.contains(&"solo-agent"));
    }

    #[test]
    fn prebuilt_difficulties_valid() {
        for c in prebuilt_challenges() {
            assert!(Challenge::validate_difficulty(c.difficulty));
        }
    }

    #[test]
    fn tower_of_balance_constraints() {
        let tob = prebuilt_challenges().into_iter().find(|c| c.id == "tower-of-balance").unwrap();
        assert!(tob.constraints.iter().any(|c| matches!(c, Constraint::MaxBlocks(50))));
        assert!(tob.constraints.iter().any(|c| matches!(c, Constraint::MinConservationScore(v) if (*v - 0.95).abs() < f64::EPSILON)));
    }

    #[test]
    fn speed_farm_has_time_limit() {
        let sf = prebuilt_challenges().into_iter().find(|c| c.id == "speed-farm").unwrap();
        assert_eq!(sf.time_limit, Some(500));
    }

    #[test]
    fn solo_agent_no_help() {
        let sa = prebuilt_challenges().into_iter().find(|c| c.id == "solo-agent").unwrap();
        assert!(sa.constraints.iter().any(|c| matches!(c, Constraint::NoAgentHelp)));
    }

    // --- Serde round-trip ---

    #[test]
    fn serde_challenge_roundtrip() {
        let c = make_challenge("serde-test", ChallengeCategory::Creativity);
        let json = serde_json::to_string(&c).unwrap();
        let back: Challenge = serde_json::from_str(&json).unwrap();
        assert_eq!(c.id, back.id);
        assert_eq!(c.category, back.category);
    }

    #[test]
    fn serde_submission_roundtrip() {
        let s = Submission::simple("c1", "alice", 42, 3.15);
        let json = serde_json::to_string(&s).unwrap();
        let back: Submission = serde_json::from_str(&json).unwrap();
        assert_eq!(s.player, back.player);
        assert!((s.score - back.score).abs() < f64::EPSILON);
    }

    #[test]
    fn serde_arena_roundtrip() {
        let mut arena = ChallengeArena::new();
        arena.create_challenge(make_challenge("c1", ChallengeCategory::Building));
        arena.submit(Submission::simple("c1", "alice", 5, 10.0));
        let json = serde_json::to_string(&arena).unwrap();
        let back: ChallengeArena = serde_json::from_str(&json).unwrap();
        assert_eq!(back.challenges.len(), 1);
        assert_eq!(back.submissions["c1"].len(), 1);
    }

    // --- Scoring rule direction ---

    #[test]
    fn fastest_time_lower_is_better() {
        let mut arena = ChallengeArena::new();
        let mut c = make_challenge("race", ChallengeCategory::Speedrun);
        c.scoring = ScoringRule::FastestTime;
        arena.create_challenge(c);
        arena.submit(Submission::simple("race", "slow", 100, 100.0));
        arena.submit(Submission::simple("race", "fast", 10, 10.0));
        let lb = arena.leaderboard("race");
        // FastestTime is ascending = lower score better
        assert_eq!(lb[0].submission.player, "fast");
    }

    #[test]
    fn highest_score_higher_is_better() {
        let mut arena = ChallengeArena::new();
        let mut c = make_challenge("score", ChallengeCategory::Building);
        c.scoring = ScoringRule::HighestScore;
        arena.create_challenge(c);
        arena.submit(Submission::simple("score", "low", 1, 10.0));
        arena.submit(Submission::simple("score", "high", 1, 90.0));
        let lb = arena.leaderboard("score");
        assert_eq!(lb[0].submission.player, "high");
    }

    #[test]
    fn leaderboard_empty_for_unknown() {
        let arena = ChallengeArena::new();
        let lb = arena.leaderboard("nope");
        assert!(lb.is_empty());
    }
}
