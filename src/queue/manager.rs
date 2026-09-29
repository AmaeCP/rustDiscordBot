use std::collections::VecDeque;
use std::fmt::Write;

use rand::seq::SliceRandom;

use crate::audio::ytdl;
use crate::error::BotError;
use crate::queue::track::QueuedTrack;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct QueueIndex(usize);

impl QueueIndex {
    #[must_use]
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    #[must_use]
    pub const fn to_zero_based(self) -> Option<usize> {
        if self.0 == 0 {
            None
        } else {
            Some(self.0 - 1)
        }
    }

    #[must_use]
    pub const fn get(self) -> usize {
        self.0
    }
}

/// Audio volume factor constrained to the supported range.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Volume(f32);

impl Volume {
    #[must_use]
    pub fn from_percent(percent: u32) -> Self {
        let factor = percent.min(200) as f32 / 100.0;
        Self(factor.clamp(0.0, 2.0))
    }

    #[must_use]
    pub const fn as_f32(self) -> f32 {
        self.0
    }
}

impl Default for Volume {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, poise::ChoiceParameter)]
pub enum LoopMode {
    #[default]
    #[name = "Off"]
    Off,
    #[name = "Track (Single)"]
    Track,
    #[name = "Queue (Entire Queue)"]
    Queue,
}

impl LoopMode {
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Off => Self::Track,
            Self::Track => Self::Queue,
            Self::Queue => Self::Off,
        }
    }

    #[must_use]
    pub const fn display(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Track => "🔂 Track",
            Self::Queue => "🔁 Queue",
        }
    }
}

#[derive(Debug)]
pub struct GuildQueue {
    pub tracks: VecDeque<QueuedTrack>,
    pub current: Option<QueuedTrack>,
    pub history: VecDeque<QueuedTrack>,
    pub handle: Option<songbird::tracks::TrackHandle>,
    pub loop_mode: LoopMode,
    pub volume: Volume,
    pub playback_generation: u64,
    pub idle_generation: u64,
    pub control_panel: Option<(
        poise::serenity_prelude::ChannelId,
        poise::serenity_prelude::MessageId,
    )>,
    pub active_channel: Option<poise::serenity_prelude::ChannelId>,
    pub paused: bool,
    pub idle_waiting: bool,
}

impl Default for GuildQueue {
    fn default() -> Self {
        Self {
            tracks: VecDeque::new(),
            current: None,
            history: VecDeque::with_capacity(50),
            handle: None,
            loop_mode: LoopMode::Off,
            volume: Volume::default(),
            playback_generation: 0,
            idle_generation: 0,
            control_panel: None,
            active_channel: None,
            paused: false,
            idle_waiting: false,
        }
    }
}

impl GuildQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_track(&mut self, track: QueuedTrack) {
        self.idle_generation = self.idle_generation.wrapping_add(1);
        self.idle_waiting = false;
        self.tracks.push_back(track);
    }

    pub fn add_playlist(&mut self, tracks: Vec<QueuedTrack>) {
        if !tracks.is_empty() {
            self.idle_generation = self.idle_generation.wrapping_add(1);
            self.idle_waiting = false;
        }
        self.tracks.extend(tracks);
    }

    pub fn remove_track(&mut self, index: QueueIndex) -> Result<QueuedTrack, BotError> {
        let zero_based = index
            .to_zero_based()
            .ok_or(BotError::InvalidIndex(index.get()))?;
        if zero_based >= self.tracks.len() {
            return Err(BotError::InvalidIndex(index.get()));
        }
        self.tracks
            .remove(zero_based)
            .ok_or(BotError::InvalidIndex(index.get()))
    }

    pub fn shuffle(&mut self) {
        let mut rng = rand::thread_rng();
        self.tracks.make_contiguous().shuffle(&mut rng);
    }

    pub fn set_volume(&mut self, vol: Volume) {
        self.volume = vol;
        if let Some(handle) = &self.handle {
            let _ = handle.set_volume(self.volume.as_f32());
        }
    }

    pub fn set_loop(&mut self, mode: LoopMode) {
        self.loop_mode = mode;
    }

    pub fn skip(&mut self) -> Option<QueuedTrack> {
        self.playback_generation = self.playback_generation.wrapping_add(1);
        self.paused = false;
        if let Some(handle) = self.handle.take() {
            let _ = handle.stop();
        }

        if let Some(prev) = &self.current {
            if self.history.len() >= 50 {
                self.history.pop_front();
            }
            self.history.push_back(prev.clone());
        }

        match self.loop_mode {
            LoopMode::Track => self.current.clone(),
            LoopMode::Queue => {
                if let Some(current) = self.current.take() {
                    self.tracks.push_back(current);
                }
                self.tracks.pop_front()
            }
            LoopMode::Off => {
                self.current = None;
                self.tracks.pop_front()
            }
        }
    }

    pub fn previous(&mut self) -> Option<QueuedTrack> {
        if self.history.is_empty() {
            return None;
        }
        self.playback_generation = self.playback_generation.wrapping_add(1);
        self.paused = false;
        if let Some(handle) = self.handle.take() {
            let _ = handle.stop();
        }

        if let Some(prev) = self.history.pop_back() {
            if let Some(current) = self.current.take() {
                self.tracks.push_front(current);
            }
            Some(prev)
        } else {
            None
        }
    }

    pub fn list_queue(&self, page: usize) -> String {
        let per_page = 10;
        let total = self.tracks.len();

        if total == 0 {
            return "Queue is empty.".to_string();
        }

        let total_pages = total.div_ceil(per_page);
        let page = page.clamp(1, total_pages);
        let start = (page - 1) * per_page;
        let end = (start + per_page).min(total);

        let mut output = String::with_capacity(512);

        for (i, track) in self.tracks.iter().enumerate().skip(start).take(end - start) {
            let _ = writeln!(
                output,
                "**{}**. {} `[{}]` — <@{}>",
                i + 1,
                track.title,
                track.duration_string(),
                track.requester
            );
        }

        let total_duration: f64 = self.tracks.iter().filter_map(|t| t.duration).sum();

        let _ = writeln!(
            output,
            "\n**Total:** {} tracks | **Duration:** {} | **Page:** {}/{}",
            total,
            ytdl::format_duration(Some(total_duration)),
            page,
            total_pages
        );

        if self.loop_mode != LoopMode::Off {
            let _ = writeln!(output, "**Loop:** {}", self.loop_mode.display());
        }

        output
    }

    pub fn clear(&mut self) {
        self.idle_generation = self.idle_generation.wrapping_add(1);
        self.playback_generation = self.playback_generation.wrapping_add(1);
        if let Some(handle) = self.handle.take() {
            let _ = handle.stop();
        }
        self.tracks.clear();
        self.current = None;
        self.paused = false;
        self.idle_waiting = false;
    }

    pub fn reset_after_disconnect(&mut self) {
        self.clear();
        self.history.clear();
        self.loop_mode = LoopMode::Off;
        self.handle = None;
        self.active_channel = None;
    }

    pub fn toggle_loop(&mut self) -> LoopMode {
        self.loop_mode = self.loop_mode.next();
        self.loop_mode
    }

    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty() && self.current.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::source::AudioSource;
    use poise::serenity_prelude as serenity;

    fn sample_track(title: &str) -> QueuedTrack {
        QueuedTrack::new(
            title.to_string(),
            Some(180.0),
            serenity::UserId::new(1),
            AudioSource::Youtube {
                url: "https://youtube.com/watch?v=test".to_string(),
            },
        )
    }

    #[test]
    fn test_loop_mode_cycle() {
        assert_eq!(LoopMode::Off.next(), LoopMode::Track);
        assert_eq!(LoopMode::Track.next(), LoopMode::Queue);
        assert_eq!(LoopMode::Queue.next(), LoopMode::Off);
    }

    #[test]
    fn test_queue_index_conversion() {
        assert_eq!(QueueIndex::new(0).to_zero_based(), None);
        assert_eq!(QueueIndex::new(1).to_zero_based(), Some(0));
        assert_eq!(QueueIndex::new(5).to_zero_based(), Some(4));
        assert_eq!(QueueIndex::new(42).get(), 42);
    }

    #[test]
    fn test_volume_clamping() {
        assert!((Volume::from_percent(0).as_f32() - 0.0).abs() < f32::EPSILON);
        assert!((Volume::from_percent(100).as_f32() - 1.0).abs() < f32::EPSILON);
        assert!((Volume::from_percent(200).as_f32() - 2.0).abs() < f32::EPSILON);
        assert!((Volume::from_percent(300).as_f32() - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_guild_queue_basic_operations() {
        let mut queue = GuildQueue::new();
        assert!(queue.is_empty());

        let t1 = sample_track("Track 1");
        let t2 = sample_track("Track 2");

        queue.add_track(t1);
        queue.add_track(t2);
        assert_eq!(queue.tracks.len(), 2);
        assert!(!queue.is_empty());

        let Ok(removed) = queue.remove_track(QueueIndex::new(1)) else {
            panic!("Expected track removal to succeed");
        };
        assert_eq!(removed.title, "Track 1");
        assert_eq!(queue.tracks.len(), 1);

        assert!(queue.remove_track(QueueIndex::new(0)).is_err());
        assert!(queue.remove_track(QueueIndex::new(99)).is_err());

        queue.clear();
        assert!(queue.is_empty());
    }

    #[test]
    fn test_guild_queue_skip_loop_modes() {
        let mut queue = GuildQueue::new();
        let t1 = sample_track("Track 1");
        let t2 = sample_track("Track 2");

        queue.add_track(t1.clone());
        queue.add_track(t2);

        queue.loop_mode = LoopMode::Off;
        queue.current = queue.tracks.pop_front();
        let next = queue.skip();
        assert_eq!(next.map(|t| t.title), Some("Track 2".to_string()));

        queue.loop_mode = LoopMode::Track;
        queue.current = Some(t1);
        let repeated = queue.skip();
        assert_eq!(repeated.map(|t| t.title), Some("Track 1".to_string()));
    }

    #[test]
    fn previous_without_history_keeps_current_playback_generation() {
        let mut queue = GuildQueue::new();
        queue.current = Some(sample_track("Current"));
        queue.playback_generation = 7;

        assert!(queue.previous().is_none());
        assert_eq!(queue.playback_generation, 7);
        assert_eq!(
            queue.current.as_ref().map(|track| track.title.as_str()),
            Some("Current")
        );
    }

    #[test]
    fn external_disconnect_resets_queue_and_voice_state() {
        let mut queue = GuildQueue::new();
        queue.current = Some(sample_track("Current"));
        queue.add_track(sample_track("Queued"));
        queue.active_channel = Some(poise::serenity_prelude::ChannelId::new(12));
        queue.history.push_back(sample_track("Previous"));

        queue.reset_after_disconnect();

        assert!(queue.is_empty());
        assert!(queue.history.is_empty());
        assert!(queue.active_channel.is_none());
    }
}
