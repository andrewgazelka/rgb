//! Audio profiling for RGB tick phases.
//!
//! Plays distinct tones for each RGB phase to help profile lag.

use std::cell::RefCell;
use std::time::Duration;

use rgb_spatial::Color;
use rodio::{OutputStream, OutputStreamHandle, Source, source::SineWave};

thread_local! {
    static AUDIO_OUTPUT: RefCell<Option<(OutputStream, OutputStreamHandle)>> = const { RefCell::new(None) };
}

/// Get or initialize the audio output stream, returning handle for playback.
fn with_audio_handle<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&OutputStreamHandle) -> R,
{
    AUDIO_OUTPUT.with_borrow_mut(|opt| {
        if opt.is_none() {
            *opt = OutputStream::try_default().ok();
        }
        opt.as_ref().map(|(_, handle)| f(handle))
    })
}

/// Frequencies for each RGB color (in Hz).
/// - Red: Lower frequency (more urgent/bass)
/// - Green: Middle frequency
/// - Blue: Higher frequency (lighter)
fn frequency_for_color(color: Color) -> f32 {
    match color {
        Color::Red => 220.0,   // A3
        Color::Green => 330.0, // E4
        Color::Blue => 440.0,  // A4
    }
}

/// Play a short beep for the given RGB color.
/// Duration is proportional to the phase duration for audible profiling.
pub fn beep_color(color: Color, duration: Duration) {
    let freq = frequency_for_color(color);

    // Scale duration - make it audible but not overwhelming
    // Min 5ms, max 50ms to avoid overlapping beeps at 20 TPS
    let beep_duration = duration.as_millis().clamp(5, 50) as u64;

    with_audio_handle(|handle| {
        let source = SineWave::new(freq)
            .take_duration(Duration::from_millis(beep_duration))
            .amplify(0.3); // Keep volume reasonable

        let _ = handle.play_raw(source.convert_samples());
    });
}

/// Play a quick tick marker for the start of an RGB phase.
pub fn tick_start(color: Color) {
    let freq = frequency_for_color(color);

    with_audio_handle(|handle| {
        // Very short click to mark phase start
        let source = SineWave::new(freq)
            .take_duration(Duration::from_millis(2))
            .amplify(0.2);

        let _ = handle.play_raw(source.convert_samples());
    });
}
