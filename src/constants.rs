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

#[derive(Copy, Clone, Debug, EnumIter)]
#[repr(u32)]
pub enum ImgAsset {
    // Grass (walkable)
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
    // Poles
    GreenPoleBottom,
    GreenPoleTop,
    BluePedestrianPoleBottom,
    BluePedestrianPoleTop,
    // Sidewalk
    Sidewalk,
    SidewalkBottomLeft,
    SidewalkBottom,
    SidewalkLeft,
    SidewalkRight,
    SidewalkTopLeft,
    SidewalkTop,
    SidewalkSpecial,
    // Sign
    Sign,
    // Trees
    TreeSmallA,
    TreeSmallB,
    // Roof
    RoofTightLeft,
    RoofTightMiddle,
    RoofTightRight,
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
            ImgAsset::GreenPoleBottom => "RPGUrbanPack/tile_0196.png",
            ImgAsset::GreenPoleTop => "RPGUrbanPack/tile_0169.png",
            ImgAsset::BluePedestrianPoleBottom => "RPGUrbanPack/tile_0195.png",
            ImgAsset::BluePedestrianPoleTop => "RPGUrbanPack/tile_0168.png",
            ImgAsset::SidewalkBottomLeft => "RPGUrbanPack/tile_0062.png",
            ImgAsset::SidewalkBottom => "RPGUrbanPack/tile_0063.png",
            ImgAsset::SidewalkLeft => "RPGUrbanPack/tile_0035.png",
            ImgAsset::SidewalkRight => "RPGUrbanPack/tile_0037.png",
            ImgAsset::SidewalkTopLeft => "RPGUrbanPack/tile_0008.png",
            ImgAsset::SidewalkTop => "RPGUrbanPack/tile_0009.png",
            ImgAsset::Sidewalk => "RPGUrbanPack/tile_0036.png",
            ImgAsset::Sign => "RPGUrbanPack/tile_0250.png",
            ImgAsset::TreeSmallA => "RPGUrbanPack/tile_0291.png",
            ImgAsset::TreeSmallB => "RPGUrbanPack/tile_0292.png",
            ImgAsset::SidewalkSpecial => "RPGUrbanPack/tile_0063.png",
            ImgAsset::RoofTightLeft => "RPGUrbanPack/tile_0138.png",
            ImgAsset::RoofTightMiddle => "RPGUrbanPack/tile_0139.png",
            ImgAsset::RoofTightRight => "RPGUrbanPack/tile_0140.png",
        }
    }
}

pub const WALKABLES: [u32; 17] = [
    ImgAsset::Grass.index(),
    ImgAsset::GrassBorderUpperLeft.index(),
    ImgAsset::GrassBorderUpper.index(),
    ImgAsset::GrassBorderUpperRight.index(),
    ImgAsset::GrassBorderLeft.index(),
    ImgAsset::Dirt.index(),
    ImgAsset::GrassBorderRight.index(),
    ImgAsset::GrassBorderLowerLeft.index(),
    ImgAsset::GrassBorderLower.index(),
    ImgAsset::GrassBorderLowerRight.index(),
    ImgAsset::Sidewalk.index(),
    ImgAsset::SidewalkBottom.index(),
    ImgAsset::SidewalkBottomLeft.index(),
    ImgAsset::SidewalkLeft.index(),
    ImgAsset::SidewalkTop.index(),
    ImgAsset::SidewalkTopLeft.index(),
    ImgAsset::SidewalkSpecial.index(),
];
