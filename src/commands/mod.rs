pub mod loop_mode;
pub mod now_playing;
pub mod pause_resume;
pub mod play;
pub mod play_local;
pub mod queue;
pub mod remove;
pub mod replay;
pub mod seek;
pub mod shuffle;
pub mod skip;
pub mod stop_leave;
pub mod volume;

use crate::types::{Data, Error};

pub fn all_commands() -> Vec<poise::Command<Data, Error>> {
    vec![
        play::play(),
        play_local::play_local(),
        queue::queue(),
        remove::remove(),
        skip::skip(),
        pause_resume::pause(),
        pause_resume::resume(),
        loop_mode::loop_cmd(),
        shuffle::shuffle(),
        volume::volume(),
        seek::seek(),
        replay::replay(),
        replay::previous(),
        stop_leave::clear(),
        stop_leave::stop(),
        stop_leave::leave(),
        now_playing::nowplaying(),
    ]
}
