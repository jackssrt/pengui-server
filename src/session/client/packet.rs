use std::sync::Arc;

use bstr::{BStr, BString};
use serde::{Deserialize, Serialize};

use crate::{
    chat::ids::MessageId,
    client::packet::{de::PacketDeserializer, error::PacketError, ser::PacketSerializer},
    player::{badge::BadgeName, ids::PlayerUuid, medal::Medals, name::PlayerName, rank::Rank},
};

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(deny_unknown_fields)]
pub enum IncomingPacket {
    #[serde(rename = "e")]
    GetExpeditions,
    #[serde(rename = "name")]
    SetName(Arc<str>),
    #[serde(rename = "say")]
    SayMap(Arc<str>),
    #[serde(rename = "psay")]
    SayParty(Arc<str>),
    #[serde(rename = "gsay")]
    SayGlobal(Arc<str>),
    #[serde(rename = "pr")]
    SetPrivateMode(u8),
    #[serde(rename = "eec")]
    ClaimExpeditionLocation { name: Arc<str>, is_free: bool },
    #[serde(rename = "i")]
    GetInfo,
    #[serde(rename = "ploc")]
    SetPlayerLocation {
        previous_map_id: u16,
        previous_locations: Arc<str>,
    },
    #[serde(rename = "lcol")]
    SetLocationColor { location_name: Arc<str> },
}
impl IncomingPacket {
    pub fn from_bstr(slice: &BStr) -> Result<Self, PacketError> {
        let mut deserializer = PacketDeserializer::new(slice, BStr::new("\u{FFFF}"));
        Self::deserialize(&mut deserializer)
    }
}

#[derive(Serialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub enum OutgoingPacket {
    #[serde(rename = "pc")]
    PlayerCount(usize),
    #[serde(rename = "i")]
    Info(String),
    #[serde(rename = "say")]
    SayMap { uuid: PlayerUuid, content: Arc<str> },
    #[serde(rename = "p")]
    PlayerInfo {
        uuid: PlayerUuid,
        name: PlayerName,
        system: Arc<str>,
        rank: Rank,
        is_authenticated: bool,
        badge: Option<BadgeName>,
        medals: Medals,
    },
    #[serde(rename = "gsay")]
    SayGlobal {
        uuid: PlayerUuid,
        // can be 0
        map_id: u16,
        previous_map_id: u16,
        previous_locations: Arc<str>,
        x: i16,
        y: i16,
        content: Arc<str>,
        message_id: MessageId,
    },
    #[serde(rename = "psay")]
    SayParty {
        player_uuid: PlayerUuid,
        content: Arc<str>,
        message_id: MessageId,
    },
    #[serde(rename = "lcol")]
    LocationColor { key: Arc<str>, value: Arc<str> },
}
impl OutgoingPacket {
    pub fn into_bstring(self) -> Result<BString, PacketError> {
        let serializer = PacketSerializer::new(BStr::new("\u{FFFF}"));
        self.serialize(serializer)
    }
}
