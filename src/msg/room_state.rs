//! A partial update to the settings of some channel.

use super::{maybe_clone, parse_bool, MessageParseError};
use crate::irc::{Command, IrcMessageRef, Tag};
use std::borrow::Cow;
use std::time::Duration;

/// A partial update to the settings of some channel.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RoomState<'src> {
  #[cfg_attr(feature = "serde", serde(borrow))]
  channel: Cow<'src, str>,

  #[cfg_attr(feature = "serde", serde(borrow))]
  channel_id: Cow<'src, str>,

  emote_only: Option<bool>,

  followers_only: Option<FollowersOnly>,

  r9k: Option<bool>,

  slow: Option<Duration>,

  subs_only: Option<bool>,
}

generate_getters! {
  <'src> for RoomState<'src> as self {
    /// Login of the channel this state was applied to.
    channel -> &str = self.channel.as_ref(),

    /// ID of the channel this state was applied to.
    channel_id -> &str = self.channel_id.as_ref(),

    /// Whether the room is in emote-only mode.
    ///
    /// Chat messages may only contain emotes.
    ///
    /// - [`None`] means no change.
    /// - [`Some`] means enabled if `true`, and disabled if `false`.
    emote_only -> Option<bool>,

    /// Whether the room is in followers-only mode.
    ///
    /// Only followers (optionally with a minimum followage) can chat.
    ///
    /// - [`None`] means no change.
    /// - [`Some`] means some change, see [`FollowersOnly`] for more information about possible values.
    followers_only -> Option<FollowersOnly>,

    /// Whether the room is in r9k mode.
    ///
    /// Only unique messages may be sent to chat.
    r9k -> Option<bool>,

    /// Whether the room is in slow mode.
    ///
    /// Users may only send messages with some minimum time between them.
    slow -> Option<Duration>,

    /// Whether the room is in subcriber-only mode.
    ///
    /// Users may only send messages if they have an active subscription.
    subs_only -> Option<bool>,
  }
}

/// Followers-only mode configuration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(
  feature = "serde",
  derive(serde::Serialize, serde::Deserialize),
  serde(rename_all = "lowercase")
)]
pub enum FollowersOnly {
  /// Followers-only mode is disabled.
  ///
  /// Anyone can send chat messages within the bounds
  /// of the other chat settings.
  Disabled,

  /// Followers-only mode is enabled, with an optional duration.
  ///
  /// If the duration is [`None`], then all followers can chat.
  /// Otherwise, only followers which have a follow age of at
  /// least the set duration can chat.
  Enabled(Option<Duration>),
}

impl<'src> RoomState<'src> {
  fn parse(message: IrcMessageRef<'src>) -> Option<Self> {
    if message.command() != Command::RoomState {
      return None;
    }

    Some(RoomState {
      channel: message.channel()?.into(),
      channel_id: message.tag(Tag::RoomId)?.into(),
      emote_only: message.tag(Tag::EmoteOnly).map(parse_bool),
      followers_only: message
        .tag(Tag::FollowersOnly)
        .and_then(|v| v.parse().ok())
        .map(|n: i64| match n {
          n if n > 0 => FollowersOnly::Enabled(Some(Duration::from_secs((n * 60) as u64))),
          0 => FollowersOnly::Enabled(None),
          _ => FollowersOnly::Disabled,
        }),
      r9k: message.tag(Tag::R9K).map(parse_bool),
      slow: message
        .tag(Tag::Slow)
        .and_then(|v| v.parse().ok())
        .map(Duration::from_secs),
      subs_only: message.tag(Tag::SubsOnly).map(parse_bool),
    })
  }

  /// Clone data to give the value a `'static` lifetime.
  pub fn into_owned(self) -> RoomState<'static> {
    RoomState {
      channel: maybe_clone(self.channel),
      channel_id: maybe_clone(self.channel_id),
      emote_only: self.emote_only,
      followers_only: self.followers_only,
      r9k: self.r9k,
      slow: self.slow,
      subs_only: self.subs_only,
    }
  }
}

impl<'src> super::FromIrc<'src> for RoomState<'src> {
  #[inline]
  fn from_irc(message: IrcMessageRef<'src>) -> Result<Self, MessageParseError> {
    Self::parse(message).ok_or(MessageParseError)
  }
}

impl<'src> From<RoomState<'src>> for super::Message<'src> {
  fn from(msg: RoomState<'src>) -> Self {
    super::Message::RoomState(msg)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_room_state_basic_full() {
    assert_irc_snapshot!(RoomState, "@emote-only=0;followers-only=-1;r9k=0;rituals=0;room-id=40286300;slow=0;subs-only=0 :tmi.twitch.tv ROOMSTATE #randers");
  }

  #[test]
  fn parse_room_state_basic_full2() {
    assert_irc_snapshot!(RoomState, "@emote-only=1;followers-only=0;r9k=1;rituals=0;room-id=40286300;slow=5;subs-only=1 :tmi.twitch.tv ROOMSTATE #randers");
  }

  #[test]
  fn parse_room_state_followers_non_zero() {
    assert_irc_snapshot!(RoomState, "@emote-only=1;followers-only=10;r9k=1;rituals=0;room-id=40286300;slow=5;subs-only=1 :tmi.twitch.tv ROOMSTATE #randers");
  }

  #[test]
  fn parse_room_state_partial_1() {
    assert_irc_snapshot!(
      RoomState,
      "@room-id=40286300;slow=5 :tmi.twitch.tv ROOMSTATE #randers"
    );
  }

  #[test]
  fn parse_room_state_partial_2() {
    assert_irc_snapshot!(
      RoomState,
      "@emote-only=1;room-id=40286300 :tmi.twitch.tv ROOMSTATE #randers"
    );
  }

  #[cfg(feature = "serde")]
  #[test]
  fn roundtrip_room_state_basic_full() {
    assert_irc_roundtrip!(RoomState, "@emote-only=0;followers-only=-1;r9k=0;rituals=0;room-id=40286300;slow=0;subs-only=0 :tmi.twitch.tv ROOMSTATE #randers");
  }

  #[cfg(feature = "serde")]
  #[test]
  fn roundtrip_room_state_basic_full2() {
    assert_irc_roundtrip!(RoomState, "@emote-only=1;followers-only=0;r9k=1;rituals=0;room-id=40286300;slow=5;subs-only=1 :tmi.twitch.tv ROOMSTATE #randers");
  }

  #[cfg(feature = "serde")]
  #[test]
  fn roundtrip_room_state_followers_non_zero() {
    assert_irc_roundtrip!(RoomState, "@emote-only=1;followers-only=10;r9k=1;rituals=0;room-id=40286300;slow=5;subs-only=1 :tmi.twitch.tv ROOMSTATE #randers");
  }

  #[cfg(feature = "serde")]
  #[test]
  fn roundtrip_room_state_partial_1() {
    assert_irc_roundtrip!(
      RoomState,
      "@room-id=40286300;slow=5 :tmi.twitch.tv ROOMSTATE #randers"
    );
  }

  #[cfg(feature = "serde")]
  #[test]
  fn roundtrip_room_state_partial_2() {
    assert_irc_roundtrip!(
      RoomState,
      "@emote-only=1;room-id=40286300 :tmi.twitch.tv ROOMSTATE #randers"
    );
  }
}
