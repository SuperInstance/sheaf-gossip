use serde::{Deserialize, Serialize};
use crate::obstruction::Obstruction;

/// A single round of gossip: messages are exchanged and H¹ is recomputed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipRound {
    pub round: usize,
    pub messages_sent: usize,
    pub h1_before: f64,
    pub h1_after: f64,
}

/// Schedule gossip frequency based on obstruction magnitude.
/// Returns the number of gossip rounds to run and the messages per round.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipSchedule {
    pub total_rounds: usize,
    pub messages_per_round: usize,
    pub urgency: f64,
}

/// Given an obstruction, determine how aggressively to gossip.
/// Higher H¹ → more rounds, more messages per round.
pub fn schedule_from_obstruction(obs: &Obstruction, num_edges: usize) -> GossipSchedule {
    let magnitude = obs.magnitude;
    let urgency = 1.0 + magnitude; // base urgency

    // More rounds for larger obstruction
    let total_rounds = if magnitude < 1e-10 {
        0
    } else {
        // Scale rounds with log of magnitude, minimum 1
        let base = (magnitude.ceil() as usize).max(1);
        // More rounds for higher dimension
        (base + obs.h1_dimension * 2).min(100)
    };

    // More messages per round for higher urgency
    let messages_per_round = if num_edges == 0 {
        0
    } else {
        (num_edges * (1 + (urgency / 2.0) as usize)).min(num_edges * 5)
    };

    GossipSchedule {
        total_rounds,
        messages_per_round,
        urgency,
    }
}

/// Compute the t-minus campaign: a countdown of gossip rounds.
/// Returns rounds with decreasing expected H¹.
pub fn t_minus_campaign(
    initial_h1: f64,
    convergence_rate: f64,
    num_rounds: usize,
) -> Vec<GossipRound> {
    let mut rounds = Vec::new();
    let mut h1 = initial_h1;
    for i in 0..num_rounds {
        let h1_before = h1;
        // Exponential decay based on convergence rate
        h1 *= (1.0 - convergence_rate).max(0.0).min(1.0);
        rounds.push(GossipRound {
            round: i + 1,
            messages_sent: num_rounds - i, // fewer messages as we converge
            h1_before,
            h1_after: h1,
        });
    }
    rounds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_obstruction_no_gossip() {
        let obs = Obstruction {
            magnitude: 0.0,
            mismatch_edges: vec![],
            h1_dimension: 0,
        };
        let sched = schedule_from_obstruction(&obs, 3);
        assert_eq!(sched.total_rounds, 0);
    }

    #[test]
    fn positive_obstruction_schedules_gossip() {
        let obs = Obstruction {
            magnitude: 5.0,
            mismatch_edges: vec![("A".into(), "B".into(), 5.0)],
            h1_dimension: 1,
        };
        let sched = schedule_from_obstruction(&obs, 3);
        assert!(sched.total_rounds > 0);
        assert!(sched.messages_per_round > 0);
        assert!(sched.urgency > 1.0);
    }

    #[test]
    fn higher_obstruction_more_rounds() {
        let low = Obstruction {
            magnitude: 1.0,
            mismatch_edges: vec![("A".into(), "B".into(), 1.0)],
            h1_dimension: 1,
        };
        let high = Obstruction {
            magnitude: 10.0,
            mismatch_edges: vec![("A".into(), "B".into(), 10.0)],
            h1_dimension: 1,
        };
        let sched_low = schedule_from_obstruction(&low, 3);
        let sched_high = schedule_from_obstruction(&high, 3);
        assert!(sched_high.total_rounds >= sched_low.total_rounds);
    }

    #[test]
    fn t_minus_campaign_decreases_h1() {
        let campaign = t_minus_campaign(10.0, 0.3, 5);
        assert_eq!(campaign.len(), 5);
        assert!((campaign[0].h1_before - 10.0).abs() < 1e-10);
        for i in 1..campaign.len() {
            assert!(campaign[i].h1_before <= campaign[i - 1].h1_before);
            assert!(campaign[i].h1_after <= campaign[i].h1_before);
        }
    }

    #[test]
    fn t_minus_campaign_rate_one_achieves_zero() {
        let campaign = t_minus_campaign(10.0, 1.0, 5);
        assert!(campaign[0].h1_after < 1e-10);
    }
}
