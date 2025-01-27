extern crate alloc;

use alloc::vec::Vec;
use std::borrow::Borrow;

pub use sci_rs::na::RealField;

#[derive(Debug, Clone, Copy)]
pub struct Ba<F: RealField + Copy> {
    /// Transfer coefficient numerator
    pub b: [F; 2],
    /// Transfer coefficient denominator
    pub a1: F,

    /// Filter delay value
    pub zi0: F,
}

///
/// Filter data through a very restricted numerator/denominator
/// aka "BA" filter.
///
pub fn lfilt_dyn<YI, F>(y: YI, ba: &mut Ba<F>) -> Vec<F>
where
    F: RealField + Copy,
    YI: IntoIterator,
    YI::Item: Borrow<F>,
{
    y.into_iter()
        .map(|yi0| {
            let x_new = *yi0.borrow() - ba.a1 * ba.zi0;
            let x = x_new * ba.b[0] + ba.zi0 * ba.b[1];
            ba.zi0 = x_new;
            x
        })
        .collect::<Vec<_>>()
}
