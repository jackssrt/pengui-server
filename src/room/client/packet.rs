use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::{
    player::{
        badge::BadgeName,
        ids::{PlayerId, PlayerUuid},
        medal::Medals,
        rank::Rank,
    },
    room::{
        client::{
            direction::Direction,
            flash::Flash,
            packet::{de::PacketDeserializer, ser::PacketSerializer},
        },
        ids::{MapId, VariableId},
    },
};
pub mod de;
pub mod error;
pub mod ser;

#[derive(Deserialize, Debug, PartialEq, Eq, Clone, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PictureData {
    pub id: u16,
    pub pos_x: i16,
    pub pos_y: i16,
    pub map_x: i16,
    pub map_y: i16,
    pub pan_x: i16,
    pub pan_y: i16,
    pub magnify: u64,
    pub top_transparency: u8,
    pub bottom_transparency: u8,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub saturation: u64,
    pub effect_mode: u64,
    pub effect_power: i64,
    pub picture_name: String,
}
#[allow(clippy::struct_excessive_bools)]
#[derive(Deserialize, Debug, PartialEq, Eq, Clone, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AddPictureData {
    pub use_transparent_color: bool,
    pub fixed_to_map: bool,
    pub spritesheet_rows: u64,
    pub spritesheet_cols: u64,
    pub spritesheet_frame: u64,
    pub spritesheet_speed: u64,
    pub spritesheet_play_once: bool,
    pub map_layer: u64,
    pub battle_layer: u64,
    pub flags: u64,
    pub blend_mode: u64,
    pub flip_x: bool,
    pub flip_y: bool,
    pub origin: u64,
}
#[derive(Serialize_repr, Deserialize_repr, Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[repr(u8)]
pub enum AnimationCommand {
    Start,
    Stop,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(deny_unknown_fields)]
pub enum IncomingPacket {
    #[serde(rename = "sr")]
    SwitchRoom(u16),
    #[serde(rename = "m")]
    Move { x: u16, y: u16 },
    #[serde(rename = "tp")]
    Teleport { x: u16, y: u16 },
    #[serde(rename = "jmp")]
    Jump { x: u16, y: u16 },
    #[serde(rename = "f")]
    ChangeFacingDirection(Direction),
    #[serde(rename = "spd")]
    ChangeSpeed(u8),
    #[serde(rename = "spr")]
    ChangeSprite { name: String, index: u32 },
    #[serde(rename = "fl")]
    PlayerFlash(Flash),
    #[serde(rename = "rfl")]
    RepeatingPlayerFlash(Flash),
    #[serde(rename = "rrfl")]
    RemoveRepeatingPlayerFlash,
    #[serde(rename = "tr")]
    ChangeTransparency(u8),
    #[serde(rename = "h")]
    ChangeSpriteVisibility { is_hidden: bool },
    #[serde(rename = "sys")]
    ChangeSystemGraphic(String),
    #[serde(rename = "se")]
    PlaySoundEffect {
        name: String,
        volume: u8,
        tempo: u16,
        balance: u8,
    },
    #[serde(rename = "ap")]
    AddPicture {
        #[serde(flatten)]
        picture_data: PictureData,
        #[serde(flatten)]
        add_picture_data: AddPictureData,
    },
    #[serde(rename = "mp")]
    MovePicture {
        #[serde(flatten)]
        picture_data: PictureData,
        duration: u64,
    },
    #[serde(rename = "rp")]
    RemovePicture(u16),
    #[serde(rename = "ba")]
    BattleAnimation(u64),
    #[serde(rename = "ss")]
    SyncSwitch { switch_id: u16, value: bool },
    #[serde(rename = "sv")]
    SyncVariable { variable_id: u16, value: u16 },
    #[serde(rename = "sev")]
    SyncEvent { is_action: bool, event_id: u32 },
    #[serde(rename = "anc")]
    AnimationCommand(AnimationCommand),
}
impl IncomingPacket {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        tracing::info!("deserializing {:x?}", bytes);
        tracing::info!("as str {}", bstr::BStr::new(bytes));
        let mut deserializer = PacketDeserializer::new(bytes.into(), b"\xef\xbf\xbf".into());
        Ok(Self::deserialize(&mut deserializer)?)
    }
}

#[derive(Serialize, Clone, Debug)]
pub enum OutgoingPacket {
    #[serde(skip)]
    Multiple(Vec<Self>),
    #[serde(rename = "s")]
    Sync {
        id: PlayerId,
        key: u32,
        uuid: PlayerUuid,
        rank: Rank,
        is_authenticated: bool,
        badge: Option<BadgeName>,
        medals: Medals,
    },
    #[serde(rename = "ri")]
    RoomId(MapId),
    #[serde(rename = "pns")]
    SyncPictureNames {
        is_prefixes: bool,
        values: Vec<String>,
    },
    #[serde(rename = "bas")]
    SyncBattleAnimations(Vec<String>),
    #[serde(rename = "anc")]
    AnimationCommand(PlayerId, AnimationCommand),
    #[serde(rename = "m")]
    Move {
        player_id: PlayerId,
        x: u16,
        y: u16,
    },
    #[serde(rename = "tp")]
    Teleport {
        player_id: PlayerId,
        x: u16,
        y: u16,
    },
    #[serde(rename = "jmp")]
    Jump {
        player_id: PlayerId,
        x: u16,
        y: u16,
    },
    #[serde(rename = "f")]
    ChangeFacingDirection {
        player_id: PlayerId,
        direction: Direction,
    },
    #[serde(rename = "spr")]
    ChangeSprite {
        player_id: PlayerId,
        name: String,
        index: u32,
    },
    #[serde(rename = "spd")]
    ChangeSpeed {
        player_id: PlayerId,
        speed: u8,
    },
    #[serde(rename = "fl")]
    PlayerFlash {
        player_id: PlayerId,
        #[serde(flatten)]
        flash: Flash,
    },
    #[serde(rename = "rfl")]
    RepeatingPlayerFlash {
        player_id: PlayerId,
        #[serde(flatten)]
        flash: Flash,
    },
    #[serde(rename = "rrfl")]
    RemoveRepeatingPlayerFlash(PlayerId),
    #[serde(rename = "tr")]
    ChangeTransparency(PlayerId, u8),
    #[serde(rename = "h")]
    ChangeSpriteVisibility {
        player_id: PlayerId,
        is_hidden: bool,
    },
    #[serde(rename = "sys")]
    ChangeSystemGraphic(PlayerId, String),
    #[serde(rename = "se")]
    PlaySoundEffect {
        player_id: PlayerId,
        name: String,
        volume: u8,
        tempo: u16,
        balance: u8,
    },
    #[serde(rename = "ba")]
    BattleAnimation(PlayerId, u64),
    SyncVariable(VariableId, u16),
    #[serde(rename = "ap")]
    AddPicture {
        #[serde(flatten)]
        picture_data: PictureData,
        #[serde(flatten)]
        add_picture_data: AddPictureData,
    },
    #[serde(rename = "mp")]
    MovePicture {
        #[serde(flatten)]
        picture_data: PictureData,
        duration: u64,
    },
}

static DELIMITER: &[u8] = b"\xEF\xBF\xBF";

impl OutgoingPacket {
    pub fn into_bytes(self) -> Result<Vec<u8>, error::PacketError> {
        self.serialize(PacketSerializer::new(DELIMITER))
    }
}
