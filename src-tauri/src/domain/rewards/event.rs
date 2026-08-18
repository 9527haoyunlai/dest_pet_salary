use super::RewardCounts;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RewardType {
    Silver,
    Gold,
    Diamond,
}

impl RewardType {
    pub fn for_event_index(event_index: u64) -> Option<Self> {
        if event_index == 0 {
            return None;
        }
        let boundary_seconds = event_index.checked_mul(10)?;
        Some(if boundary_seconds % 3_600 == 0 {
            Self::Diamond
        } else if boundary_seconds % 60 == 0 {
            Self::Gold
        } else {
            Self::Silver
        })
    }

    pub fn as_code(self) -> &'static str {
        match self {
            Self::Silver => "SILVER",
            Self::Gold => "GOLD",
            Self::Diamond => "DIAMOND",
        }
    }

    pub fn counts(self) -> RewardCounts {
        match self {
            Self::Silver => RewardCounts {
                silver: 1,
                gold: 0,
                diamond: 0,
            },
            Self::Gold => RewardCounts {
                silver: 0,
                gold: 1,
                diamond: 0,
            },
            Self::Diamond => RewardCounts {
                silver: 0,
                gold: 0,
                diamond: 1,
            },
        }
    }
}
