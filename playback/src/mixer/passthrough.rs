use super::VolumeGetter;
use super::{MappedCtrl, VolumeCtrl};
use super::{Mixer, MixerConfig};
use librespot_core::Error;
use portable_atomic::AtomicU64;
use std::sync::Arc;
use std::sync::atomic::Ordering;

#[derive(Clone)]
pub struct PassthroughMixer {
    // Store volume state to track Spotify app volume changes
    // but don't use it for audio gain (always use 1.0)
    volume: Arc<AtomicU64>,
    volume_ctrl: VolumeCtrl,
}

impl Mixer for PassthroughMixer {
    fn open(config: MixerConfig) -> Result<Self, Error> {
        let volume_ctrl = config.volume_ctrl;
        info!("Mixing with passthrough and volume control: {volume_ctrl:?}");

        Ok(Self {
            volume: Arc::new(AtomicU64::new(f64::to_bits(0.5))),
            volume_ctrl,
        })
    }

    fn volume(&self) -> u16 {
        let mapped_volume = f64::from_bits(self.volume.load(Ordering::Relaxed));
        self.volume_ctrl.as_unmapped(mapped_volume)
    }

    fn set_volume(&self, volume: u16) {
        // Store volume state to update Spotify app UI
        // but don't apply it to audio stream
        let mapped_volume = self.volume_ctrl.to_mapped(volume);
        self.volume
            .store(mapped_volume.to_bits(), Ordering::Relaxed)
    }

    fn get_soft_volume(&self) -> Box<dyn VolumeGetter + Send> {
        // Always return 1.0 (100% gain) for audio stream
        Box::new(PassthroughVolume)
    }
}

impl PassthroughMixer {
    pub const NAME: &'static str = "passthrough";
}

struct PassthroughVolume;

impl VolumeGetter for PassthroughVolume {
    #[inline]
    fn attenuation_factor(&self) -> f64 {
        1.0 // Always 100% gain - no attenuation
    }
}
