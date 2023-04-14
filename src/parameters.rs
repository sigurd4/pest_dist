use std::f32::MAX_EXP;

use vst::prelude::PluginParameters;
use vst::util::AtomicFloat;

pub const MIN_LOG: f32 = 0.001;
pub const MAX_LOG: f32 = 1.0;

#[derive(Clone, Copy)]
pub enum Control
{
    Gain,
    Filter,
    Volume
}

impl Control
{
    pub const CONTROLS: [Self; 3] = [
        Self::Gain,
        Self::Filter,
        Self::Volume
    ];

    pub fn from(i: i32) -> Self
    {
        Self::CONTROLS[i as usize]
    }
}

pub struct RatDistortionParameters
{
    pub gain: AtomicFloat,
    pub filter: AtomicFloat,
    pub volume: AtomicFloat
}

impl PluginParameters for RatDistortionParameters
{
    fn get_parameter_label(&self, index: i32) -> String
    {
        match Control::from(index)
        {
            Control::Gain => "%".to_string(),
            Control::Filter => "%".to_string(),
            Control::Volume => "%".to_string()
        }
    }

    fn get_parameter_text(&self, index: i32) -> String
    {
        match Control::from(index)
        {
            Control::Gain => format!("{:.3}", self.get_parameter(index)),
            Control::Filter => format!("{:.3}", self.get_parameter(index)),
            Control::Volume => format!("{:.3}", self.get_parameter(index))
        }
    }

    fn get_parameter_name(&self, index: i32) -> String
    {
        match Control::from(index)
        {
            Control::Gain => "Gain".to_string(),
            Control::Filter => "Filter".to_string(),
            Control::Volume => "Volume".to_string()
        }
    }

    /// Get the value of parameter at `index`. Should be value between 0.0 and 1.0.
    fn get_parameter(&self, index: i32) -> f32
    {
        match Control::from(index)
        {
            Control::Gain => (self.gain.get().log10() - MIN_LOG.log10())/(MAX_LOG.log10() - MIN_LOG.log10()),
            Control::Filter => (self.filter.get().log10() - MIN_LOG.log10())/(MAX_LOG.log10() - MIN_LOG.log10()),
            Control::Volume => (self.volume.get().log10() - MIN_LOG.log10())/(MAX_LOG.log10() - MIN_LOG.log10())
        }
    }
    
    fn set_parameter(&self, index: i32, value: f32)
    {
        match Control::from(index)
        {
            Control::Gain => self.gain.set(10.0f32.powf(value*(MAX_LOG.log10() - MIN_LOG.log10()) + MIN_LOG.log10())),
            Control::Filter => self.filter.set(10.0f32.powf(value*(MAX_LOG.log10() - MIN_LOG.log10()) + MIN_LOG.log10())),
            Control::Volume => self.volume.set(10.0f32.powf(value*(MAX_LOG.log10() - MIN_LOG.log10()) + MIN_LOG.log10()))
        }
    }

    fn change_preset(&self, preset: i32) {}

    fn get_preset_num(&self) -> i32 {
        0
    }

    fn set_preset_name(&self, name: String) {}

    fn get_preset_name(&self, preset: i32) -> String {
        "".to_string()
    }

    fn can_be_automated(&self, index: i32) -> bool {
        index < Control::CONTROLS.len() as i32
    }

    fn get_preset_data(&self) -> Vec<u8> {
        [
            self.gain.get().to_le_bytes().to_vec(),
            self.filter.get().to_le_bytes().to_vec(),
            self.volume.get().to_le_bytes().to_vec()
        ].concat()
    }

    fn get_bank_data(&self) -> Vec<u8>
    {
        self.get_preset_data()
    }

    fn load_preset_data(&self, data: &[u8])
    {
        let mut i = 0;
        self.gain.set(f32::from_le_bytes(*data[i..].split_array_ref().0)); i += 4;
        self.filter.set(f32::from_le_bytes(*data[i..].split_array_ref().0)); i += 4;
        self.volume.set(f32::from_le_bytes(*data[i..].split_array_ref().0)); i += 4;
    }

    fn load_bank_data(&self, data: &[u8])
    {
        self.load_preset_data(data)
    }
}