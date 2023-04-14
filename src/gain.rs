use crate::*;

#[derive(Clone, Copy)]
pub struct GainFilter
{
    pub gain: f32,
    pub w: [f32; 3]
}

impl GainFilter
{
    pub fn new(gain: f32) -> Self
    {
        Self {
            gain,
            w: [0.0; 3]
        }
    }
}

impl IIRFilter<3, 1> for GainFilter
{
    fn a(&self, rate: f32) -> [f32; 4]
    {
        let r_g = RPOT*self.gain;

        let a3 = R1*R2*C1*C2*C3*r_g;
        let a2 = R2*C2*C3*r_g + R1*C1*(R2*C2 + C3*r_g);
        let a1 = (R1*C1 + R2*C2) + C3*r_g;
        let a0 = 1.0;

        let rate2 = rate*rate;
        let rate3 = rate2*rate;

        [
            8.0*rate3*a3 + 4.0*rate2*a2 + 2.0*rate*a1 + a0,
            -24.0*rate3*a3 - 4.0*rate2*a2 + 2.0*rate*a1 + 3.0*a0,
            24.0*rate3*a3 - 4.0*rate2*a2 - 2.0*rate*a1 + 3.0*a0,
            -8.0*rate3*a3 + 4.0*rate2*a2 - 2.0*rate*a1 + a0,
        ]
    }
    fn b(&self, rate: f32) -> [[f32; 4]; 1]
    {
        let r_g = RPOT*self.gain;

        let b3 = R1*R2*C1*C2*C3*r_g;
        let b2 = ((R1 + R2)*r_g + R1*R2)*C1*C2 + (R1*C1 + R2*C2)*C3*r_g;
        let b1 = (C1 + C2 + C3)*r_g + (R1*C1 + R2*C2);
        let b0 = 1.0;

        let rate2 = rate*rate;
        let rate3 = rate2*rate;

        [
            [
                8.0*rate3*b3 + 4.0*rate2*b2 + 2.0*rate*b1 + b0,
                -24.0*rate3*b3 - 4.0*rate2*b2 + 2.0*rate*b1 + 3.0*b0,
                24.0*rate3*b3 - 4.0*rate2*b2 - 2.0*rate*b1 + 3.0*b0,
                -8.0*rate3*b3 + 4.0*rate2*b2 - 2.0*rate*b1 + b0,
            ]
        ]
    }
    fn w(&mut self) -> &mut [f32; 3]
    {
        &mut self.w
    }
}