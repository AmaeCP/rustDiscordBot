use std::collections::VecDeque;
use std::fmt::Write;

use rand::seq::SliceRandom;

use crate::audio::ytdl;
use crate::error::BotError;
use crate::queue::track::QueuedTrack;

#[derive(Debug, Clone, Copy, PartialEq, poise::ChoiceParameter)]
pub enum LoopMode {
    #[name = "Off"]
    Off,
    #[name = "Track (Single)"]
    Track,
    #[name = "Queue (Entire Queue)"]
    Queue,
}

impl LoopMode {
    pub fn next(&self) -> Self {
        match self {
            Self::Off => Self::Track,
            Self::Track => Self::Queue,
            Self::Queue => Self::Off,
        }
    }

    pub fn display(&self) -> &str {
        match self {
            Self::Off => "Off",
            Self::Track => "🔂 Track",
            Self::Queue => "🔁 Queue",
        }
    }
}

pub struct GuildQueue {
    pub tracks: VecDeque<QueuedTrack>,
    pub current: Option<QueuedTrack>,
    pub history: VecDeque<QueuedTrack>,
    pub handle: Option<songbird::tracks::TrackHandle>,
    pub loop_mode: LoopMode,
    pub volume: f32,
}

impl GuildQueue {
    pub fn new() -> Self {
        Self {
            tracks: VecDeque::new(),
            current: None,
            history: VecDeque::with_capacity(50),
            handle: None,
            loop_mode: LoopMode::Off,
            volume: 1.0,
        }
    }

    pub fn add_track(&mut self, track: QueuedTrack) {
        self.tracks.push_back(track);
    }

    pub fn add_playlist(&mut self, tracks: Vec<QueuedTrack>) {
        self.tracks.extend(tracks);
    }

    pub fn remove_track(&mut self, index: usize) -> Result<QueuedTrack, BotError> {
        if index == 0 || index > self.tracks.len() {
            return Err(BotError::InvalidIndex(index));
        }
        self.tracks
            .remove(index - 1)
            .ok_or(BotError::InvalidIndex(index))
    }

    pub fn shuffle(&mut self) {
        let mut rng = rand::thread_rng();
        self.tracks.make_contiguous().shuffle(&mut rng);
    }

    pub fn set_volume(&mut self, vol: f32) {
        self.volume = vol.clamp(0.0, 2.0);
        if let Some(handle) = &self.handle {
            let _ = handle.set_volume(self.volume);
        }
    }

    pub fn set_loop(&mut self, mode: LoopMode) {
        self.loop_mode = mode;
    }

    pub fn skip(&mut self) -> Option<QueuedTrack> {
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

        let total_duration: f64 = self
            .tracks
            .iter()
            .filter_map(|t| t.duration)
            .sum();

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
        if let Some(handle) = self.handle.take() {
            let _ = handle.stop();
        }
        self.tracks.clear();
        self.current = None;
    }

    pub fn toggle_loop(&mut self) -> &LoopMode {
        self.loop_mode = self.loop_mode.next();
        &self.loop_mode
    }

    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty() && self.current.is_none()
    }
}
