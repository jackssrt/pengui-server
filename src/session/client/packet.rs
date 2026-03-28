use bstr::{BStr, BString};
use serde::{Deserialize, Serialize};

use crate::{
    client::packet::{de::PacketDeserializer, error::PacketError, ser::PacketSerializer},
    locations::Locations,
    player::{badge::BadgeName, ids::PlayerUuid, medal::Medals, name::PlayerName, rank::Rank},
    room::ids::MapId,
};

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(deny_unknown_fields)]
pub enum IncomingPacket {
    #[serde(rename = "e")]
    GetExpeditions,
    #[serde(rename = "name")]
    SetName(String),
    #[serde(rename = "say")]
    SayMap(String),
    #[serde(rename = "psay")]
    SayParty(String),
    #[serde(rename = "gsay")]
    SayGlobal(String),
    #[serde(rename = "pr")]
    SetPrivateMode(u8),
    #[serde(rename = "eec")]
    ClaimExpeditionLocation { name: String, is_free: bool },
    #[serde(rename = "i")]
    Info(),
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
    SayMap { uuid: PlayerUuid, content: String },
    #[serde(rename = "p")]
    PlayerInfo {
        uuid: PlayerUuid,
        name: PlayerName,
        system: String,
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
        previous_locations: Locations,
        x: i16,
        y: i16,
        content: String,
        message_id: String,
    },
    #[serde(rename = "psay")]
    SayParty {
        player_uuid: PlayerUuid,
        content: String,
        message_id: String,
    },
}
impl OutgoingPacket {
    pub fn into_bstring(self) -> Result<BString, PacketError> {
        let serializer = PacketSerializer::new(BStr::new("\u{FFFF}"));
        self.serialize(serializer)
    }
}
