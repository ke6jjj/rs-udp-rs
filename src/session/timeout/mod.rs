use crate::datasource::Channel;
use tokio::time::{Duration, Instant};

pub struct ChannelState {
    pub channel: Channel,
    pub as_of: Option<Instant>,
    pub alive: Option<bool>,
}

pub struct ChannelChecker {
    timeout: Option<Duration>,
    channel_states: Vec<ChannelState>,
}

impl<'a> ChannelChecker {
    pub fn new_for_timeout(timeout: Option<Duration>) -> Self {
        Self {
            timeout,
            channel_states: Vec::new(),
        }
    }

    pub fn track_channel(&mut self, channel: Channel) {
        for channel_state in self.channel_states.iter_mut() {
            if channel_state.channel == channel {
                // Already tracking this channel
                return;
            }
        }
        let new_state = ChannelState {
            channel,
            as_of: None,
            alive: None,
        };
        self.channel_states.push(new_state);
    }

    pub fn start(&mut self, when: Instant) {
        for channel_state in self.channel_states.iter_mut() {
            channel_state.as_of = Some(when);
        }
    }

    fn oldest_not_dead(&self) -> Option<Instant> {
        let mut oldest_not_dead = None::<Instant>;
        for channel in self.channel_states.iter() {
            if channel.alive.unwrap_or(true) {
                if let Some(oldest_time) = oldest_not_dead {
                    // Assertion: Caller must have "started" this checker.
                    // If not, this will cause a panic.
                    if oldest_time <= channel.as_of.unwrap() {
                        continue;
                    }
                }
                oldest_not_dead = channel.as_of
            }
        }
        oldest_not_dead
    }

    // Returns true if the channel was already "alive". A return of false
    // indicates that the caller should probably broadcast the good news
    // that the channel is available.
    pub fn mark_channel_alive(&mut self, when: Instant, channel: Channel) -> bool {
        for channel_state in self.channel_states.iter_mut() {
            if channel_state.channel == channel {
                channel_state.as_of.replace(when);
                return channel_state.alive.replace(true).unwrap_or(false);
            }
        }
        // We weren't configured to monitor this channel. Any answer is
        // acceptable here.
        true
    }

    // Returns the minimum duration that the caller should wait in order to
    // determine if any channel has stopped producing data.
    pub fn next_timeout(&self, from: Instant) -> Option<Duration> {
        if let Some(timeout) = self.timeout {
            if let Some(oldest) = self.oldest_not_dead() {
                let elapsed = from.duration_since(oldest);
                if elapsed > timeout {
                    // Deadline already exceeded
                    return Some(Duration::ZERO);
                } else {
                    return Some(timeout - elapsed);
                }
            }
        }
        // No timeout is configured and/or nothing is currently alive.
        None
    }

    // Notes that no channel activity has been detected as of a certain time,
    // and returns an iterator over all channels that have now timed out
    // as a result.
    pub fn timeout_iter(&'a mut self, now: Instant) -> TimeoutIter<'a> {
        // Not to be called unless you know there's a timeout configured.
        TimeoutIter {
            timeout_point: now - self.timeout.unwrap(),
            channel_state_iter: self.channel_states.iter_mut(),
        }
    }
}

pub struct TimeoutIter<'a> {
    timeout_point: Instant,
    channel_state_iter: core::slice::IterMut<'a, ChannelState>,
}

impl<'a> Iterator for TimeoutIter<'a> {
    type Item = &'a ChannelState;

    fn next(&mut self) -> Option<Self::Item> {
        for channel_state in self.channel_state_iter.by_ref() {
            if channel_state.alive.unwrap_or(true)
                && channel_state.as_of.unwrap() < self.timeout_point
            {
                channel_state.alive.replace(false);
                return Some(&*channel_state);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::Channel;
    use super::ChannelChecker;
    use tokio::time::Instant;

    // If no timeout is configured then no timeout is expected.
    #[test]
    fn test_one() {
        let channel = Channel::try_from("EHZ").expect("channel");
        let now = Instant::now();
        let mut checker = ChannelChecker::new_for_timeout(None);
        checker.track_channel(channel);
        checker.start(now);
        assert_eq!(checker.next_timeout(Instant::now()), None);
    }

    // If no channels are being tracked then no timeout is expected.
    #[test]
    fn test_two() {
        let now = Instant::now();
        let mut checker = ChannelChecker::new_for_timeout(None);
        checker.start(now);
        assert_eq!(checker.next_timeout(Instant::now()), None);
    }

    #[test]
    fn test_three() {
        let now = Instant::now();
        let timeout = Duration::from_secs(5);
        let mut checker = ChannelChecker::new_for_timeout(Some(timeout));
        checker.track_channel(Channel::Ehz);
        checker.start(now);
        let result = checker.next_timeout(now + Duration::from_secs(1));
        assert!(result.is_some());
        assert_eq!(result.unwrap(), timeout - Duration::from_secs(1));
    }

    #[test]
    fn unmonitored_channel_ok() {
        let now = Instant::now();
        let timeout = Duration::from_secs(5);
        let mut checker = ChannelChecker::new_for_timeout(Some(timeout));
        checker.track_channel(Channel::Ehz);
        checker.start(now);
        checker.mark_channel_alive(now + Duration::from_secs(2), Channel::Enn);
    }
}
