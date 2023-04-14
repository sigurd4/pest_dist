#![feature(adt_const_params)]
#![feature(const_for)]
#![feature(const_fn_floating_point_arithmetic)]
#![feature(const_mut_refs)]
#![feature(split_array)]
#![feature(const_eval_limit)]

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use real_time_fir_iir_filters::iir::{FirstOrderRCFilter, IIRFilter};
use vst::{prelude::*, plugin_main};

use self::gain::GainFilter;
use self::parameters::{RatDistortionParameters, Control, MIN_LOG};

pub mod parameters;
pub mod gain;

struct RatDistortionPlugin
{
    pub param: Arc<RatDistortionParameters>,
    pub input_filter1: [FirstOrderRCFilter; CHANNEL_COUNT],
    pub input_filter2: [FirstOrderRCFilter; CHANNEL_COUNT],
    pub gain_filter: [GainFilter; CHANNEL_COUNT],
    pub clip_filter: [FirstOrderRCFilter; CHANNEL_COUNT],
    pub filter: [FirstOrderRCFilter; CHANNEL_COUNT],
    pub output_filter1: [FirstOrderRCFilter; CHANNEL_COUNT],
    pub output_filter2: [FirstOrderRCFilter; CHANNEL_COUNT],
    pub clip: Box<[f32; CLIP_N]>,
    pub rate: f32
}

pub const CLIP_MIN: f32 = -4.5;
pub const CLIP_MAX: f32 = 4.5;
pub const CLIP_N: usize = 32768;

impl RatDistortionPlugin
{
    pub fn clip_gen() -> Box<[f32; CLIP_N]>
    {
        let mut c = Box::new([0.0; CLIP_N]);
        for i in 0..CLIP_N
        {
            let x = i as f32/(CLIP_N - 1) as f32*(CLIP_MAX - CLIP_MIN) + CLIP_MIN;
            c[i] = x + x.signum()*(I_0*R_D -  fastapprox::faster::lambertw(I_0*R_D*ALPHA*((I_0*R_D + x.abs())*ALPHA).exp())/ALPHA)
        }
        let min = c.into_iter().filter(|c| c.is_finite()).reduce(|a, b| a.min(b)).unwrap_or(0.0);
        let max = c.into_iter().filter(|c| c.is_finite()).reduce(|a, b| a.max(b)).unwrap_or(0.0);
        for i in 0..CLIP_N
        {
            if !c[i].is_finite()
            {
                c[i] = if i >= CLIP_N/2 {max} else {min}
            }
        }
        c
    }

    pub fn clip(&self, x: f32) -> f32
    {
        let i = ((x - CLIP_MIN)/(CLIP_MAX - CLIP_MIN)*(CLIP_N - 1) as f32).min((CLIP_N - 1) as f32).max(0.0);
        let i0 = (i.floor() as usize).min(CLIP_N - 1);
        let i1 = (i.ceil() as usize).min(CLIP_N - 1);
        let f = i - i.floor();
        self.clip[i0]*(1.0 - f) + self.clip[i1]*f
    }
}

const CHANNEL_COUNT: usize = 2;

const K: f32 = 1.38e-23;
const R_D: f32 = 1000.0;
const I_0: f32 = 0.000000004;
const Q_E: f32 = 1.602176634e-19;
const ETA: f32 = 2.0;
const T: f32 = 20.0 + 273.15;
const ALPHA: f32 = Q_E/ETA/K/T;

const R1: f32 = 47.0;
const R2: f32 = 560.0;
const RPOT: f32 = 100000.0;
const C1: f32 = 0.0000022;
const C2: f32 = 0.0000047;
const C3: f32 = 0.000000000100;

const I_DSS: f32 = 0.006;
const V_GS_OFF: f32 = -4.0; //-1.0 - -7.0
const R_J: f32 = 10000.0;

impl Plugin for RatDistortionPlugin
{
    fn new(_host: HostCallback) -> Self
    where
        Self: Sized
    {
        RatDistortionPlugin {
            param: Arc::new(RatDistortionParameters {
                gain: AtomicFloat::from(0.1),
                filter: AtomicFloat::from(0.1),
                volume: AtomicFloat::from(1.0)
            }),
            input_filter1: [FirstOrderRCFilter::new(1000000.0, 0.000000022); CHANNEL_COUNT],
            input_filter2: [FirstOrderRCFilter::new(1000.0, 0.000000001); CHANNEL_COUNT],
            gain_filter: [GainFilter::new(0.1); CHANNEL_COUNT],
            clip_filter: [FirstOrderRCFilter::new(R_D, 0.0000047); CHANNEL_COUNT],
            filter: [FirstOrderRCFilter::new(1500.0, 0.0000000033); CHANNEL_COUNT],
            output_filter1: [FirstOrderRCFilter::new(1000000.0, 0.000000022); CHANNEL_COUNT],
            output_filter2: [FirstOrderRCFilter::new(110000.0, 0.000001); CHANNEL_COUNT],
            clip: Self::clip_gen(),
            rate: 44100.0
        }
    }

    fn get_info(&self) -> Info
    {
        Info {
            name: "Pest Distortion".to_string(),
            vendor: "Soma FX".to_string(),
            presets: 0,
            parameters: Control::CONTROLS.len() as i32,
            inputs: CHANNEL_COUNT as i32,
            outputs: CHANNEL_COUNT as i32,
            midi_inputs: 0,
            midi_outputs: 0,
            unique_id: 453635633,
            version: 1,
            category: Category::Effect,
            initial_delay: 0,
            preset_chunks: false,
            f64_precision: true,
            silent_when_stopped: true,
            ..Default::default()
        }
    }

    fn set_sample_rate(&mut self, rate: f32)
    {
        self.rate = rate;
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>)
    {
        let gain = self.param.gain.get();
        let filter = self.param.filter.get();
        let volume = self.param.volume.get();

        for (c, (input_channel, output_channel)) in buffer.zip().enumerate()
        {
            self.gain_filter[c].gain = gain;
            self.filter[c].r = 1500.0 + RPOT*filter;
            for (input_sample, output_sample) in input_channel.into_iter()
                .zip(output_channel.into_iter())
            {
                let x = self.gain_filter[c].filter(self.rate,
                    self.input_filter2[c].filter(self.rate,
                        self.input_filter1[c].filter(self.rate, *input_sample)[1]
                    )[0]
                )[0];
                let x_c = self.clip(x);
                let xd = self.clip_filter[c].filter(self.rate, x-x_c)[1] + x_c;
                let z = self.output_filter1[c].filter(self.rate,
                    self.filter[c].filter(self.rate, self.clip(xd))[0]
                )[1];
                let y0 = V_GS_OFF*((V_GS_OFF*V_GS_OFF/I_DSS/R_J - 4.0*V_GS_OFF)/I_DSS/R_J).sqrt()/2.0 - V_GS_OFF;
                let y = V_GS_OFF*((V_GS_OFF*V_GS_OFF/I_DSS/R_J - 4.0*V_GS_OFF + 4.0*z)/I_DSS/R_J).sqrt()/2.0 - V_GS_OFF + z - y0;
                *output_sample = self.output_filter2[c].filter(self.rate, y)[1]*volume
            }
        }
    }

    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters>
    {
        self.param.clone()
    }
}

plugin_main!(RatDistortionPlugin);

pub mod tests {
    use vst::prelude::{Plugin, HostCallback, AudioBuffer};

    use crate::{RatDistortionPlugin};

    #[test]
    fn test()
    {
        let mut plugin = RatDistortionPlugin::new(HostCallback::default());

        let i: [f32; 10] = [0.0; 10];
        let mut o: [f32; 10] = [0.0; 10];

        unsafe
        {
            let pi: *const *const f32 = &i.as_ptr();
            let po: *mut *mut f32 = &mut o.as_mut_ptr();
            let mut ab = AudioBuffer::from_raw(2, 2, pi, po, 10);
            plugin.process(&mut ab);
        }

        //println!("{}", plugin.clip.map(|c| format!("{:.3}", c)).join(","))
    }
}