//! Time-based, interruptible motion system for the Herdr TUI.
//!
//! The module is intentionally decoupled from rendering and from the Tokio runtime.
//! Transitions are advanced by elapsed time only, never by frame count.  This keeps
//! motion deterministic and cheap over SSH or under output pressure.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use ratatui::layout::Rect;

/// Capability of the connected terminal as reported by the backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalCapability {
    /// Truecolor / 24-bit color.
    TrueColor,
    /// 256-color palette.
    Color256,
    /// 16-color palette.
    Color16,
    /// Monochrome or unknown; avoid color-only cues.
    Monochrome,
}

impl TerminalCapability {
    /// Choose the most capable known level from a set of booleans.
    pub fn from_flags(truecolor: bool, color256: bool, color16: bool) -> Self {
        if truecolor {
            Self::TrueColor
        } else if color256 {
            Self::Color256
        } else if color16 {
            Self::Color16
        } else {
            Self::Monochrome
        }
    }
}

/// User/system motion preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotionPreference {
    Full,
    Reduced,
}

/// Clock abstraction so tests can advance time manually.
pub trait Clock: Send + Sync {
    fn now(&self) -> Instant;
}

/// Production clock backed by `Instant::now`.
pub struct RealClock;

impl Clock for RealClock {
    fn now(&self) -> Instant {
        Instant::now()
    }
}

impl Default for RealClock {
    fn default() -> Self {
        Self
    }
}

/// Easing functions. All operate on a normalized `t` in `[0.0, 1.0]` and return
/// a value in the same range.
pub struct Easing;

impl Easing {
    pub fn linear(t: f32) -> f32 {
        t.clamp(0.0, 1.0)
    }

    pub fn ease_out(t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        1.0 - (1.0 - t).powi(2)
    }

    pub fn ease_in_out(t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        if t < 0.5 {
            2.0 * t * t
        } else {
            1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
        }
    }

    /// Critically damped spring-like easing: smooth, no overshoot.
    pub fn smooth(t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }
}

/// Affected UI region for a transition. The scheduler uses this to cancel or
/// coalesce related transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiRegion {
    Sidebar,
    SidebarMode,
    TabBar,
    FleetOps,
    Settings,
    Launcher,
    Modal,
    Toast,
    PaneBorder,
    PaneChrome,
    Scrollbar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterruptionPolicy {
    /// Reverse from the current interpolated value when interrupted.
    Retarget,
    /// Snap to the final value immediately when interrupted.
    Snap,
}

/// A scalar transition between two values.
#[derive(Debug, Clone)]
pub struct Transition {
    pub region: UiRegion,
    pub started_at: Instant,
    pub duration: Duration,
    pub from: f32,
    pub to: f32,
    pub easing: fn(f32) -> f32,
    pub interruption: InterruptionPolicy,
}

impl Transition {
    /// Create a new transition.
    pub fn new(
        region: UiRegion,
        started_at: Instant,
        duration: Duration,
        from: f32,
        to: f32,
        easing: fn(f32) -> f32,
        interruption: InterruptionPolicy,
    ) -> Self {
        Self {
            region,
            started_at,
            duration,
            from,
            to,
            easing,
            interruption,
        }
    }

    /// Sample the transition at a given instant. Returns `(value, finished)`.
    /// When `finished` is true, callers should remove the transition.
    pub fn sample(&self, now: Instant) -> (f32, bool) {
        if self.duration == Duration::ZERO {
            return (self.to, true);
        }
        let elapsed = now.duration_since(self.started_at);
        if elapsed >= self.duration {
            return (self.to, true);
        }
        let t = (elapsed.as_secs_f32() / self.duration.as_secs_f32()).clamp(0.0, 1.0);
        let v = (self.easing)(t);
        let value = self.from + (self.to - self.from) * v;
        (value, false)
    }

    /// Current target value.
    pub fn target(&self) -> f32 {
        self.to
    }
}

/// A rectangular transition, useful for layout regions that move/resize.
#[derive(Debug, Clone)]
pub struct RectTransition {
    pub region: UiRegion,
    pub started_at: Instant,
    pub duration: Duration,
    pub from: Rect,
    pub to: Rect,
    pub easing: fn(f32) -> f32,
    pub interruption: InterruptionPolicy,
}

impl RectTransition {
    pub fn sample(&self, now: Instant) -> (Rect, bool) {
        if self.duration == Duration::ZERO {
            return (self.to, true);
        }
        let elapsed = now.duration_since(self.started_at);
        if elapsed >= self.duration {
            return (self.to, true);
        }
        let t = (elapsed.as_secs_f32() / self.duration.as_secs_f32()).clamp(0.0, 1.0);
        let v = (self.easing)(t);
        let lerp = |a: u16, b: u16| {
            if a <= b {
                a + ((b - a) as f32 * v) as u16
            } else {
                b + ((a - b) as f32 * (1.0 - v)) as u16
            }
        };
        let rect = Rect::new(
            lerp(self.from.x, self.to.x),
            lerp(self.from.y, self.to.y),
            lerp(self.from.width, self.to.width),
            lerp(self.from.height, self.to.height),
        );
        (rect, false)
    }
}

/// Central transition registry / scheduler.
///
/// Keeps a bounded set of active transitions. It does not spawn tasks; the
/// event loop ticks it with the current time and asks for the next wake-up.
#[derive(Debug, Default)]
pub struct TransitionScheduler {
    scalar: HashMap<UiRegion, Transition>,
    rects: Vec<RectTransition>,
}

impl TransitionScheduler {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register or replace a scalar transition for a region.
    pub fn set(&mut self, transition: Transition) {
        self.scalar.insert(transition.region, transition);
    }

    /// Register or replace a rectangular transition.
    pub fn set_rect(&mut self, transition: RectTransition) {
        self.rects.retain(|r| r.region != transition.region);
        self.rects.push(transition);
    }

    /// Cancel any transitions for a region and snap them to their target values.
    /// Returns the scalar value (if any) that should be considered final.
    pub fn cancel(&mut self, region: UiRegion) -> Option<f32> {
        self.scalar.remove(&region).map(|t| t.target())
    }

    /// Remove completed transitions and advance all others. Returns the next
    /// useful wake-up instant, or `None` if no transitions are active.
    pub fn advance(&mut self, now: Instant) -> Option<Instant> {
        self.scalar.retain(|_, t| t.sample(now).1 == false);
        self.rects.retain(|t| t.sample(now).1 == false);

        let mut next: Option<Instant> = None;
        for t in self.scalar.values() {
            let finish = t.started_at + t.duration;
            next = Some(next.map_or(finish, |n| n.min(finish)));
        }
        for t in &self.rects {
            let finish = t.started_at + t.duration;
            next = Some(next.map_or(finish, |n| n.min(finish)));
        }
        next
    }

    /// Sample a scalar transition at the given time.
    pub fn sample_scalar(&self, region: UiRegion, now: Instant) -> Option<f32> {
        self.scalar.get(&region).map(|t| t.sample(now).0)
    }

    /// Sample a rectangular transition at the given time.
    pub fn sample_rect(&self, region: UiRegion, now: Instant) -> Option<Rect> {
        self.rects
            .iter()
            .find(|t| t.region == region)
            .map(|t| t.sample(now).0)
    }

    /// True if any transition is still active.
    pub fn is_active(&self) -> bool {
        !self.scalar.is_empty() || !self.rects.is_empty()
    }

    /// True if there is an transition for the given region.
    pub fn has_region(&self, region: UiRegion) -> bool {
        self.scalar.contains_key(&region) || self.rects.iter().any(|t| t.region == region)
    }
}

/// Motion policy derived from configuration and environment.
#[derive(Debug, Clone, Copy)]
pub struct MotionPolicy {
    pub preference: MotionPreference,
    pub max_fps: u8,
    pub remote_max_fps: u8,
    pub capability: TerminalCapability,
}

impl Default for MotionPolicy {
    fn default() -> Self {
        Self {
            preference: MotionPreference::Full,
            max_fps: 60,
            remote_max_fps: 30,
            capability: TerminalCapability::TrueColor,
        }
    }
}

impl MotionPolicy {
    /// Effective FPS given the current session context.
    pub fn effective_fps(&self, is_remote: bool) -> u8 {
        if is_remote {
            self.remote_max_fps.min(self.max_fps)
        } else {
            self.max_fps
        }
    }

    /// Interval between allowed animation frames for the current context.
    pub fn frame_interval(&self, is_remote: bool) -> Duration {
        let fps = self.effective_fps(is_remote).max(1);
        Duration::from_nanos(1_000_000_000 / u64::from(fps))
    }

    /// Duration should resolve immediately when reduced motion is requested.
    pub fn resolve_duration(&self, desired: Duration) -> Duration {
        match self.preference {
            MotionPreference::Reduced => Duration::ZERO,
            MotionPreference::Full => desired,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ManualClock {
        now: std::sync::Mutex<Instant>,
    }

    impl Clock for ManualClock {
        fn now(&self) -> Instant {
            *self.now.lock().unwrap()
        }
    }

    #[test]
    fn transition_reaches_target_and_reports_finished() {
        let start = Instant::now();
        let t = Transition::new(
            UiRegion::Sidebar,
            start,
            Duration::from_millis(100),
            0.0,
            10.0,
            Easing::linear,
            InterruptionPolicy::Retarget,
        );
        assert_eq!(t.sample(start).0, 0.0);
        assert!(!t.sample(start).1);
        let (v, finished) = t.sample(start + Duration::from_millis(150));
        assert!(finished);
        assert_eq!(v, 10.0);
    }

    #[test]
    fn ease_in_out_is_symmetric() {
        let t = Easing::ease_in_out(0.0);
        assert!((t - 0.0).abs() < f32::EPSILON);
        let t = Easing::ease_in_out(0.5);
        assert!((t - 0.5).abs() < f32::EPSILON);
        let t = Easing::ease_in_out(1.0);
        assert!((t - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn scheduler_returns_next_wake_time() {
        let start = Instant::now();
        let mut sched = TransitionScheduler::new();
        sched.set(Transition::new(
            UiRegion::Sidebar,
            start,
            Duration::from_millis(100),
            0.0,
            1.0,
            Easing::linear,
            InterruptionPolicy::Retarget,
        ));
        let next = sched.advance(start);
        assert_eq!(next, Some(start + Duration::from_millis(100)));
        assert!(sched.is_active());

        sched.advance(start + Duration::from_millis(200));
        assert!(!sched.is_active());
        assert!(sched.sample_scalar(UiRegion::Sidebar, start).is_none());
    }

    #[test]
    fn reduced_motion_resolves_duration_to_zero() {
        let mut policy = MotionPolicy::default();
        policy.preference = MotionPreference::Reduced;
        assert_eq!(policy.resolve_duration(Duration::from_millis(100)), Duration::ZERO);
    }

    #[test]
    fn rect_transition_interpolates_rect() {
        let start = Instant::now();
        let t = RectTransition {
            region: UiRegion::Sidebar,
            started_at: start,
            duration: Duration::from_millis(100),
            from: Rect::new(0, 0, 10, 10),
            to: Rect::new(10, 20, 30, 40),
            easing: Easing::linear,
            interruption: InterruptionPolicy::Retarget,
        };
        let (rect, _) = t.sample(start + Duration::from_millis(50));
        assert_eq!(rect.x, 5);
        assert_eq!(rect.y, 10);
        assert_eq!(rect.width, 20);
        assert_eq!(rect.height, 25);
    }
}
