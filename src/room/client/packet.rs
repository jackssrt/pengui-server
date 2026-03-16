use anyhow::Result;
use serde::{Deserialize, Serialize};

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
        id: u16,
        pos_x: i16,
        pos_y: i16,
        map_x: u16,
        map_y: u16,
        pan_x: i16,
        pan_y: i16,
        magnify: u64,
        top_transparency: u8,
        bottom_transparency: u8,
        red: u8,
        green: u8,
        blue: u8,
        saturation: u64,
        effect_mode: u64,
        effect_power: u64,
        picture_name: String,
        use_transparent_color: bool,
        fixed_to_map: bool,
        spritesheet_rows: u64,
        spritesheet_cols: u64,
        spritesheet_frame: u64,
        spritesheet_speed: u64,
        spritesheet_play_once: bool,
        map_layer: u64,
        battle_layer: u64,
        flags: u64,
        blend_mode: u64,
        flip_x: bool,
        flip_y: bool,
        origin: u64,
    },
    #[serde(rename = "mp")]
    MovePicture {
        id: u16,
        pos_x: i16,
        pos_y: i16,
        map_x: i16,
        map_y: i16,
        pan_x: i16,
        pan_y: i16,
        magnify: u64,
        top_transparency: u8,
        bottom_transparency: u8,
        red: u8,
        green: u8,
        blue: u8,
        saturation: u64,
        effect_mode: u64,
        effect_power: u64,
        picture_name: String,
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
    AnimationCommand,
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
    AnimationCommand,
    #[serde(rename = "m")]
    Move { player_id: PlayerId, x: u16, y: u16 },
    #[serde(rename = "tp")]
    Teleport { player_id: PlayerId, x: u16, y: u16 },
    #[serde(rename = "jmp")]
    Jump { player_id: PlayerId, x: u16, y: u16 },
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
    ChangeSpeed { player_id: PlayerId, speed: u8 },
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
}

static DELIMITER: &[u8] = b"\xEF\xBF\xBF";

impl OutgoingPacket {
    pub fn into_bytes(self) -> Result<Vec<u8>, error::PacketError> {
        self.serialize(PacketSerializer::new(DELIMITER))
    }
}
