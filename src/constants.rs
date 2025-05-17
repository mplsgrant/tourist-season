#![allow(unused)]

use bevy::prelude::*;
use strum_macros::EnumIter;

pub const Z_TILEMAP: i32 = 0;

pub const BITCOIN_DIR: &str = "bitcoind";
pub const MAP_DIR: &str = "map";
pub const MAP_JSON: &str = "map.json";

/// Marks an entity as being a Popup.
/// Current use: tilemap interactions query to see if the node with this marker is displayed and if it is displayed, the system disables tilemap interaction.
/// In other words, don't register clicks to the tilemap if the Popup is visible to the user.
#[derive(Component, Clone)]
pub struct PopupBase;

// pub const GRASS_PATH: &str = "tiles-test/tile_0000.png";
// pub const GRASS_IDX: u32 = 0;
// pub const GRASS_BORDER_UPPER_LEFT_PATH: &str = "tiles-test/tile_0012.png";
// pub const GRASS_BORDER_UPPER_LEFT_IDX: u32 = 1;
// pub const GRASS_BORDER_UPPER: &str = "tiles-test/tile_0013.png";
// pub const GRASS_BORDER_UPPER_IDX: u32 = 2;
// pub const GRASS_BORDER_UPPER_RIGHT: &str = "tiles-test/tile_0014.png";
// pub const GRASS_BORDER_UPPER_RIGHT_IDX: u32 = 3;
// pub const GRASS_BORDER_LEFT: &str = "tiles-test/tile_0024.png";
// pub const GRASS_BORDER_LEFT_IDX: u32 = 4;
// pub const DIRT: &str = "tiles-test/tile_0025.png";
// pub const DIRT_IDX: u32 = 5;
// pub const GRASS_BORDER_RIGHT: &str = "tiles-test/tile_0026.png";
// pub const GRASS_BORDER_RIGHT_IDX: u32 = 6;
// pub const GRASS_BORDER_LOWER_LEFT: &str = "tiles-test/tile_0036.png";
// pub const GRASS_BORDER_LOWER_LEFT_IDX: u32 = 7;
// pub const GRASS_BORDER_LOWER: &str = "tiles-test/tile_0037.png";
// pub const GRASS_BORDER_LOWER_IDX: u32 = 8;
// pub const GRASS_BORDER_LOWER_RIGHT: &str = "tiles-test/tile_0038.png";
// pub const GRASS_BORDER_LOWER_RIGHT_IDX: u32 = 9;

// pub const RED_BRICK_COL_UPPER: &str = "RPGUrbanPack/tile_0016.png";
// pub const RED_BRICK_COL_MID_A: &str = "RPGUrbanPack/tile_0043.png";
// pub const RED_BRICK_COL_MID_B: &str = "RPGUrbanPack/tile_0070.png";
// pub const RED_BRICK_COL_LOWER: &str = "RPGUrbanPack/tile_0097.png";

// pub const RED_BRICK_BLANK_A: &str = "RPGUrbanPack/tile_0072.png";
// pub const RED_BRICK_BLANK_B: &str = "RPGUrbanPack/tile_0074.png";
// pub const RED_BRICK_BLANK_C: &str = "RPGUrbanPack/tile_0075.png";
// pub const RED_BRICK_BLANK_D: &str = "RPGUrbanPack/tile_0076.png";
// pub const RED_BRICK_BLANK_LOWER_A: &str = "RPGUrbanPack/tile_0099.png";
// pub const RED_BRICK_BLANK_LOWER_B: &str = "RPGUrbanPack/tile_0101.png";
// pub const RED_BRICK_BLANK_LOWER_C: &str = "RPGUrbanPack/tile_0102.png";
// pub const RED_BRICK_BLANK_LOWER_D: &str = "RPGUrbanPack/tile_0103.png";

// pub const RED_BRICK_LEFT_UPPER: &str = "RPGUrbanPack/tile_0017.png";
// pub const RED_BRICK_MID_UPPER_A: &str = "RPGUrbanPack/tile_0018.png";
// pub const RED_BRICK_MID_UPPER_B: &str = "RPGUrbanPack/tile_0020.png";
// pub const RED_BRICK_MID_UPPER_C: &str = "RPGUrbanPack/tile_0021.png";
// pub const RED_BRICK_MID_UPPER_D: &str = "RPGUrbanPack/tile_0022.png";
// pub const RED_BRICK_RIGHT_UPPER: &str = "RPGUrbanPack/tile_0019.png";
// pub const RED_BRICK_LEFT_MID: &str = "RPGUrbanPack/tile_0071.png";
// pub const RED_BRICK_RIGHT_MID: &str = "RPGUrbanPack/tile_0073.png";

// pub const DOOR_SINGLE_GLASS_CLOSED: &str = "RPGUrbanPack/tile_0281.png";
// pub const DOOR_SINGLE_GLASS_OPEN: &str = "RPGUrbanPack/tile_0308.png";

// pub const DOOR_SINGLE_RED_CLOSED: &str = "RPGUrbanPack/tile_0282.png";
// pub const DOOR_SINGLE_RED_OPEN: &str = "RPGUrbanPack/tile_0309.png";

// pub const DOOR_SINGLE_YELLOW_CLOSED: &str = "RPGUrbanPack/tile_0283.png";
// pub const DOOR_SINGLE_YELLOW_OPEN: &str = "RPGUrbanPack/tile_0310.png";

// pub const DOOR_DOUBLE_YELLOW_CLOSED: &str = "RPGUrbanPack/tile_0284.png";
// pub const DOOR_DOUBLE_YELLOW_OPEN: &str = "RPGUrbanPack/tile_0311.png";

// pub const DOOR_DOUBLE_GLASS_CLOSED: &str = "RPGUrbanPack/tile_0285.png";
// pub const DOOR_DOUBLE_GLASS_OPEN: &str = "RPGUrbanPack/tile_0312.png";

// pub const DOOR_DOUBLE_SILVER_CLOSED: &str = "RPGUrbanPack/tile_0339.png";
// pub const DOOR_DOUBLE_SILVER_OPEN: &str = "RPGUrbanPack/tile_0366.png";

// pub const WINDOW_CREAM_DECORATIVE_NARROW: &str = "RPGUrbanPack/tile_0335.png";
// pub const WINDOW_CREAM_DECORATIVE_WIDE: &str = "RPGUrbanPack/tile_0390.png";

// pub const WINDOW_CREAM_ROUNDED_UPPER: &str = "RPGUrbanPack/tile_0336.png";
// pub const WINDOW_CREAM_BEVELED_UPPER: &str = "RPGUrbanPack/tile_0337.png";
// pub const WINDOW_CREAM_SQUARE_UPPER: &str = "RPGUrbanPack/tile_0338.png";
// pub const WINDOW_CREAM_SQUARE_MID: &str = "RPGUrbanPack/tile_0365.png";
// pub const WINDOW_CREAM_SQUARE_LOWER: &str = "RPGUrbanPack/tile_0392.png";

// pub const WINDOW_CREAM_SLIDING_UPPER: &str = "RPGUrbanPack/tile_0362.png";
// pub const WINDOW_CREAM_SLIDING_LOWER: &str = "RPGUrbanPack/tile_0389.png";
// pub const WINDOW_CREAM_SLIDING_ALONE_LITTLE: &str = "RPGUrbanPack/tile_0363.png";
// pub const WINDOW_CREAM_SLIDING_ALONE_BIG: &str = "RPGUrbanPack/tile_0391.png";

// pub const WINDOW_CREAM_PANE_ALONE: &str = "RPGUrbanPack/tile_0364.png";

// pub const BLUE_ROOF_UPPER_LEFT: &str = "RPGUrbanPack/tile_0081.png";

#[derive(Copy, Clone, Debug, EnumIter)]
#[repr(u32)]
pub enum ImgAsset {
    // Grass
    Grass,
    GrassBorderUpperLeft,
    GrassBorderUpper,
    GrassBorderUpperRight,
    GrassBorderLeft,
    Dirt,
    GrassBorderRight,
    GrassBorderLowerLeft,
    GrassBorderLower,
    GrassBorderLowerRight,
    // Red brick column
    RedBrickColUpper,
    RedBrickColMidA,
    RedBrickColMidB,
    RedBrickColLower,
    // Blank red brick
    RedBrickBlankA,
    RedBrickBlankB,
    RedBrickBlankC,
    RedBrickBlankD,
    RedBrickBlankLowerA,
    RedBrickBlankLowerB,
    RedBrickBlankLowerC,
    RedBrickBlankLowerD,
    // Decorated red brick
    RedBrickLeftUpper,
    RedBrickMidUpperA,
    RedBrickMidUpperB,
    RedBrickMidUpperC,
    RedBrickMidUpperD,
    RedBrickRightUpper,
    RedBrickLeftMid,
    RedBrickRightMid,
    // Single glass door
    DoorSingleGlassClosed,
    DoorSingleGlassOpen,
    // Single red door
    DoorSingleRedClosed,
    DoorSingleRedOpen,
    // Single yellow door
    DoorSingleYellowClosed,
    DoorSingleYellowOpen,
    // Double yellow door
    DoorDoubleYellowClosed,
    DoorDoubleYellowOpen,
    // Double glass door
    DoorDoubleGlassClosed,
    DoorDoubleGlassOpen,
    // Double silver door
    DoorDoubleSilverClosed,
    DoorDoubleSilverOpen,
    // Cream decorative window
    WindowCreamDecorativeNarrow,
    WindowCreamDecorativeWide,
    // Cream composite windows
    WindowCreamRoundedUpper,
    WindowCreamBeveledUpper,
    WindowCreamSquareUpper,
    WindowCreamSquareMid,
    WindowCreamSquareLower,
    // Cream sliding windows
    WindowCreamSlidingUpper,
    WindowCreamSlidingLower,
    WindowCreamSlidingAloneLittle,
    WindowCreamSlidingAloneBig,
    // Cream large window
    WindowCreamPaneAlone,
    // Blue roof
    BlueRoofUpperLeft,
    // Green tourist
    GreenTouristStandingLeft,
    GreenTouristStandingFront,
    GreenTouristStandingBack,
    GreenTouristStandingRight,
    GreenTouristWalkingLeftA,
    GreenTouristWalkingFrontA,
    GreenTouristWalkingBackA,
    GreenTouristWalkingRightA,
    GreenTouristWalkingLeftB,
    GreenTouristWalkingFrontB,
    GreenTouristWalkingBackB,
    GreenTouristWalkingRightB,
}

impl ImgAsset {
    pub const fn index(self) -> u32 {
        self as u32
    }

    pub const fn path(self) -> &'static str {
        match self {
            ImgAsset::Grass => "tiles-test/tile_0000.png",
            ImgAsset::GrassBorderUpperLeft => "tiles-test/tile_0012.png",
            ImgAsset::GrassBorderUpper => "tiles-test/tile_0013.png",
            ImgAsset::GrassBorderUpperRight => "tiles-test/tile_0014.png",
            ImgAsset::GrassBorderLeft => "tiles-test/tile_0024.png",
            ImgAsset::Dirt => "tiles-test/tile_0025.png",
            ImgAsset::GrassBorderRight => "tiles-test/tile_0026.png",
            ImgAsset::GrassBorderLowerLeft => "tiles-test/tile_0036.png",
            ImgAsset::GrassBorderLower => "tiles-test/tile_0037.png",
            ImgAsset::GrassBorderLowerRight => "tiles-test/tile_0038.png",
            ImgAsset::RedBrickColUpper => "RPGUrbanPack/tile_0016.png",
            ImgAsset::RedBrickColMidA => "RPGUrbanPack/tile_0043.png",
            ImgAsset::RedBrickColMidB => "RPGUrbanPack/tile_0070.png",
            ImgAsset::RedBrickColLower => "RPGUrbanPack/tile_0097.png",
            ImgAsset::RedBrickBlankA => "RPGUrbanPack/tile_0072.png",
            ImgAsset::RedBrickBlankB => "RPGUrbanPack/tile_0074.png",
            ImgAsset::RedBrickBlankC => "RPGUrbanPack/tile_0075.png",
            ImgAsset::RedBrickBlankD => "RPGUrbanPack/tile_0076.png",
            ImgAsset::RedBrickBlankLowerA => "RPGUrbanPack/tile_0099.png",
            ImgAsset::RedBrickBlankLowerB => "RPGUrbanPack/tile_0101.png",
            ImgAsset::RedBrickBlankLowerC => "RPGUrbanPack/tile_0102.png",
            ImgAsset::RedBrickBlankLowerD => "RPGUrbanPack/tile_0103.png",
            ImgAsset::RedBrickLeftUpper => "RPGUrbanPack/tile_0017.png",
            ImgAsset::RedBrickMidUpperA => "RPGUrbanPack/tile_0018.png",
            ImgAsset::RedBrickMidUpperB => "RPGUrbanPack/tile_0020.png",
            ImgAsset::RedBrickMidUpperC => "RPGUrbanPack/tile_0021.png",
            ImgAsset::RedBrickMidUpperD => "RPGUrbanPack/tile_0022.png",
            ImgAsset::RedBrickRightUpper => "RPGUrbanPack/tile_0019.png",
            ImgAsset::RedBrickLeftMid => "RPGUrbanPack/tile_0071.png",
            ImgAsset::RedBrickRightMid => "RPGUrbanPack/tile_0073.png",
            ImgAsset::DoorSingleGlassClosed => "RPGUrbanPack/tile_0281.png",
            ImgAsset::DoorSingleGlassOpen => "RPGUrbanPack/tile_0308.png",
            ImgAsset::DoorSingleRedClosed => "RPGUrbanPack/tile_0282.png",
            ImgAsset::DoorSingleRedOpen => "RPGUrbanPack/tile_0309.png",
            ImgAsset::DoorSingleYellowClosed => "RPGUrbanPack/tile_0283.png",
            ImgAsset::DoorSingleYellowOpen => "RPGUrbanPack/tile_0310.png",
            ImgAsset::DoorDoubleYellowClosed => "RPGUrbanPack/tile_0284.png",
            ImgAsset::DoorDoubleYellowOpen => "RPGUrbanPack/tile_0311.png",
            ImgAsset::DoorDoubleGlassClosed => "RPGUrbanPack/tile_0285.png",
            ImgAsset::DoorDoubleGlassOpen => "RPGUrbanPack/tile_0312.png",
            ImgAsset::DoorDoubleSilverClosed => "RPGUrbanPack/tile_0339.png",
            ImgAsset::DoorDoubleSilverOpen => "RPGUrbanPack/tile_0366.png",
            ImgAsset::WindowCreamDecorativeNarrow => "RPGUrbanPack/tile_0335.png",
            ImgAsset::WindowCreamDecorativeWide => "RPGUrbanPack/tile_0390.png",
            ImgAsset::WindowCreamRoundedUpper => "RPGUrbanPack/tile_0336.png",
            ImgAsset::WindowCreamBeveledUpper => "RPGUrbanPack/tile_0337.png",
            ImgAsset::WindowCreamSquareUpper => "RPGUrbanPack/tile_0338.png",
            ImgAsset::WindowCreamSquareMid => "RPGUrbanPack/tile_0365.png",
            ImgAsset::WindowCreamSquareLower => "RPGUrbanPack/tile_0392.png",
            ImgAsset::WindowCreamSlidingUpper => "RPGUrbanPack/tile_0362.png",
            ImgAsset::WindowCreamSlidingLower => "RPGUrbanPack/tile_0389.png",
            ImgAsset::WindowCreamSlidingAloneLittle => "RPGUrbanPack/tile_0363.png",
            ImgAsset::WindowCreamSlidingAloneBig => "RPGUrbanPack/tile_0391.png",
            ImgAsset::WindowCreamPaneAlone => "RPGUrbanPack/tile_0364.png",
            ImgAsset::BlueRoofUpperLeft => "RPGUrbanPack/tile_0081.png",
            ImgAsset::GreenTouristStandingLeft => "RPGUrbanPack/tile_0023.png",
            ImgAsset::GreenTouristStandingFront => "RPGUrbanPack/tile_0024.png",
            ImgAsset::GreenTouristStandingBack => "RPGUrbanPack/tile_0025.png",
            ImgAsset::GreenTouristStandingRight => "RPGUrbanPack/tile_0026.png",
            ImgAsset::GreenTouristWalkingLeftA => "RPGUrbanPack/tile_0050.png",
            ImgAsset::GreenTouristWalkingFrontA => "RPGUrbanPack/tile_0051.png",
            ImgAsset::GreenTouristWalkingBackA => "RPGUrbanPack/tile_0052.png",
            ImgAsset::GreenTouristWalkingRightA => "RPGUrbanPack/tile_0053.png",
            ImgAsset::GreenTouristWalkingLeftB => "RPGUrbanPack/tile_0077.png",
            ImgAsset::GreenTouristWalkingFrontB => "RPGUrbanPack/tile_0078.png",
            ImgAsset::GreenTouristWalkingBackB => "RPGUrbanPack/tile_0079.png",
            ImgAsset::GreenTouristWalkingRightB => "RPGUrbanPack/tile_0080.png",
        }
    }
}
