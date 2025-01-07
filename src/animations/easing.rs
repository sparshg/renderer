use std::f32::consts::PI;

pub trait Easing {
    fn ease(&self, t: f32) -> f32;
}

macro_rules! define_easing {
    ($name:ident, $func:expr) => {
        pub struct $name;

        impl Easing for $name {
            fn ease(&self, t: f32) -> f32 {
                $func(t)
            }
        }
    };
}

define_easing!(Linear, |t: f32| t);

// from https://easings.net/
define_easing!(EaseInSine, |t: f32| 1. - (1. - t).cos());
define_easing!(EaseOutSine, |t: f32| t.sin());
define_easing!(EaseInOutSine, |t: f32| 0.5 - (0.5 * t).cos());
define_easing!(EaseInQuad, |t: f32| t * t);
define_easing!(EaseOutQuad, |t: f32| t * (2. - t));
define_easing!(EaseInOutQuad, |t: f32| match t {
    ..0.5 => 2. * t * t,
    _ => -1. + (4. - 2. * t) * t,
});

define_easing!(EaseInCubic, |t: f32| t * t * t);
define_easing!(EaseOutCubic, |t: f32| (t - 1.).powi(3) + 1.);
define_easing!(EaseInOutCubic, |t: f32| match t {
    ..0.5 => 4. * t * t * t,
    _ => (t - 1.).powi(3) * 4. + 1.,
});

define_easing!(EaseInQuart, |t: f32| t * t * t * t);
define_easing!(EaseOutQuart, |t: f32| 1. - (t - 1.).powi(4));
define_easing!(EaseInOutQuart, |t: f32| match t {
    ..0.5 => 8. * t * t * t * t,
    _ => 1. - (t - 1.).powi(4) * 8.,
});

define_easing!(EaseInQuint, |t: f32| t * t * t * t * t);
define_easing!(EaseOutQuint, |t: f32| 1. + (t - 1.).powi(5));
define_easing!(EaseInOutQuint, |t: f32| match t {
    ..0.5 => 16. * t * t * t * t * t,
    _ => 1. + (t - 1.).powi(5) * 16.,
});

define_easing!(EaseInExpo, |t: f32| match t {
    0. => 0.,
    _ => 2.0f32.powf(10. * (t - 1.)),
});
define_easing!(EaseOutExpo, |t: f32| match t {
    1. => 1.,
    _ => 1. - 2.0f32.powf(-10. * t),
});
define_easing!(EaseInOutExpo, |t: f32| match t {
    0. => 0.,
    1. => 1.,
    ..0.5 => 2.0f32.powf(10. * (2. * t - 1.)) / 2.,
    _ => (2. - 2.0f32.powf(-10. * (2. * t - 1.))) / 2.,
});

define_easing!(EaseInCirc, |t: f32| 1. - (1. - t * t).sqrt());
define_easing!(EaseOutCirc, |t: f32| (1. - (t - 1.).powi(2)).sqrt());
define_easing!(EaseInOutCirc, |t: f32| match t {
    ..0.5 => (1. - (1. - 4. * t * t).sqrt()) / 2.,
    _ => ((1. - (1. - (2. * t - 2.).powi(2)).sqrt()) + 1.) / 2.,
});

define_easing!(EaseInBack, |t: f32| t * t * (2.70158 * t - 1.70158));
define_easing!(EaseOutBack, |t: f32| 1.
    + (t - 1.).powi(2) * (2.70158 * (t - 1.) + 1.70158));
define_easing!(EaseInOutBack, |t: f32| match t {
    ..0.5 => (2. * t).powi(2) * (3.5949095 * 2. * t - 2.5949095) / 2.,
    _ => ((2. * t - 2.).powi(2) * (3.5949095 * (t * 2. - 2.) + 2.5949095) + 2.) / 2.,
});

define_easing!(EaseInElastic, |t: f32| match t {
    0. => 0.,
    1. => 1.,
    _ => -2.0f32.powf(10. * (t - 1.)) * (2. * PI * (t - 1.75)).sin(),
});
define_easing!(EaseOutElastic, |t: f32| match t {
    0. => 0.,
    1. => 1.,
    _ => 2.0f32.powf(-10. * t) * (2. * PI * (t - 0.75)).sin() + 1.,
});
define_easing!(EaseInOutElastic, |t: f32| match t {
    0. => 0.,
    1. => 1.,
    ..0.5 => -(2.0f32.powf(10. * (2. * t - 1.)) * (2. * PI * (2. * t - 1.1125)).sin()) / 2.,
    _ => (2.0f32.powf(-10. * (2. * t - 1.)) * (2. * PI * (2. * t - 1.1125)).sin()) / 2. + 1.,
});

define_easing!(EaseInBounce, |t: f32| 1. - EaseOutBounce.ease(1. - t));
define_easing!(EaseOutBounce, |t: f32| match t {
    t if t < 1. / 2.75 => 7.5625 * t * t,
    t if t < 2. / 2.75 => 7.5625 * (t - 1.5 / 2.75) * (t - 1.5 / 2.75) + 0.75,
    t if t < 2.5 / 2.75 => 7.5625 * (t - 2.25 / 2.75) * (t - 2.25 / 2.75) + 0.9375,
    _ => 7.5625 * (t - 2.625 / 2.75) * (t - 2.625 / 2.75) + 0.984375,
});
define_easing!(EaseInOutBounce, |t: f32| match t {
    ..0.5 => EaseInBounce.ease(t * 2.) * 0.5,
    _ => EaseOutBounce.ease(t * 2. - 1.) * 0.5 + 0.5,
});

// from manim
define_easing!(Smooth, |t: f32| t * t * t * (10. + 6. * t * t - 15. * t));
define_easing!(RushInto, |t: f32| 2. * Smooth.ease(0.5 * t));
define_easing!(RushFrom, |t: f32| 2. * Smooth.ease(0.5 * (t + 1.)) - 1.);
define_easing!(SlowInto, |t: f32| (1. - (1. - t).powi(2)).sqrt());
define_easing!(DoubleSmooth, |t: f32| match t {
    ..0.5 => 0.5 * Smooth.ease(2. * t),
    _ => 0.5 * (1. + Smooth.ease(2. * t - 1.)),
});
define_easing!(ThereAndBack, |t: f32| Smooth.ease(1. - (2. * t - 1.).abs()));

pub struct Wiggle(pub f32);

impl Easing for Wiggle {
    fn ease(&self, t: f32) -> f32 {
        ThereAndBack.ease(t) * (self.0 * PI * t).sin()
    }
}

pub struct ExponentialDecay(pub f32);

impl Easing for ExponentialDecay {
    fn ease(&self, t: f32) -> f32 {
        1.0 - (-t / self.0).exp()
    }
}
