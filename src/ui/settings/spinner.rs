use crate::config::SpinnerStyle;

pub(crate) struct SpinnerCategory {
    pub label: &'static str,
    pub styles: &'static [SpinnerStyle],
}

pub(crate) const SPINNER_CATEGORIES: &[SpinnerCategory] = &[
    SpinnerCategory {
        label: "classic",
        styles: &[
            SpinnerStyle::Dots,
            SpinnerStyle::DotsFull,
            SpinnerStyle::DotsCorner,
            SpinnerStyle::Arc,
            SpinnerStyle::Circle,
            SpinnerStyle::CircleQuarters,
            SpinnerStyle::CircleHalves,
            SpinnerStyle::SquareCorners,
            SpinnerStyle::Triangle,
            SpinnerStyle::Star,
            SpinnerStyle::Star2,
            SpinnerStyle::Arrow,
            SpinnerStyle::Arrow3,
            SpinnerStyle::Bounce,
            SpinnerStyle::BoxBounce,
            SpinnerStyle::Pipe,
        ],
    },
    SpinnerCategory {
        label: "motion",
        styles: &[
            SpinnerStyle::Noise,
            SpinnerStyle::Aesthetic,
            SpinnerStyle::GrowVertical,
            SpinnerStyle::GrowHorizontal,
            SpinnerStyle::Point,
            SpinnerStyle::BetaWave,
            SpinnerStyle::Layer,
            SpinnerStyle::Liquid,
            SpinnerStyle::Crystal,
            SpinnerStyle::Galaxy,
            SpinnerStyle::Vortex,
            SpinnerStyle::Toggle,
            SpinnerStyle::Flip,
            SpinnerStyle::Sandwich,
            SpinnerStyle::BouncingBar,
            SpinnerStyle::BouncingBall,
        ],
    },
    SpinnerCategory {
        label: "play",
        styles: &[
            SpinnerStyle::Pong,
            SpinnerStyle::Shark,
            SpinnerStyle::Fish,
            SpinnerStyle::Binary,
            SpinnerStyle::DotsCircle,
            SpinnerStyle::Sand,
            SpinnerStyle::Dots8Bit,
            SpinnerStyle::Moon,
            SpinnerStyle::Clock,
            SpinnerStyle::Earth,
            SpinnerStyle::Weather,
            SpinnerStyle::Hearts,
            SpinnerStyle::Balloon,
            SpinnerStyle::Grenade,
            SpinnerStyle::FingerDance,
            SpinnerStyle::FistBump,
        ],
    },
    SpinnerCategory {
        label: "emoji",
        styles: &[
            SpinnerStyle::Smiley,
            SpinnerStyle::Monkey,
            SpinnerStyle::Speaker,
            SpinnerStyle::Runner,
            SpinnerStyle::SoccerHeader,
            SpinnerStyle::Mindblown,
            SpinnerStyle::OrangePulse,
            SpinnerStyle::BluePulse,
            SpinnerStyle::OrangeBluePulse,
            SpinnerStyle::TimeTravel,
            SpinnerStyle::Christmas,
            SpinnerStyle::Flame,
            SpinnerStyle::Pizza,
            SpinnerStyle::Dizzy,
            SpinnerStyle::Ninja,
            SpinnerStyle::Magic,
        ],
    },
    SpinnerCategory {
        label: "fantasy",
        styles: &[
            SpinnerStyle::Robot,
            SpinnerStyle::Boom,
            SpinnerStyle::Unicorn,
            SpinnerStyle::Bee,
            SpinnerStyle::Dragon,
            SpinnerStyle::Ghost,
            SpinnerStyle::Pumpkin,
            SpinnerStyle::Wizard,
            SpinnerStyle::Crown,
            SpinnerStyle::Diamond,
            SpinnerStyle::Fire,
            SpinnerStyle::Rocket,
            SpinnerStyle::StarSpin,
            SpinnerStyle::Confetti,
            SpinnerStyle::Cthulhu,
            SpinnerStyle::DwarfFortress,
        ],
    },
];

pub(crate) fn active_spinner_category(index: usize) -> &'static SpinnerCategory {
    SPINNER_CATEGORIES
        .get(index)
        .unwrap_or(&SPINNER_CATEGORIES[0])
}

pub(crate) fn spinner_style_count() -> usize {
    SpinnerStyle::ALL.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spinner_categories_cover_all_styles() {
        let categorized: usize = SPINNER_CATEGORIES.iter().map(|c| c.styles.len()).sum();
        assert_eq!(categorized, spinner_style_count());
    }
}
