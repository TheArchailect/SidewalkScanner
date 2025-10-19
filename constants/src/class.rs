/// Road classification codes for heightmap generation
pub const ROAD_CLASSIFICATIONS: &[u8] = &[2, 10, 11, 12];

pub struct ClassInfo {
    pub id: u8,
    pub name: &'static str,
}

pub const CLASS_MAP: &[ClassInfo] = &[
    ClassInfo {
        id: 0,
        name: "unclassified",
    },
    ClassInfo {
        id: 2,
        name: "ground, sidewalk",
    },
    ClassInfo {
        id: 3,
        name: "vegetation - low",
    },
    ClassInfo {
        id: 4,
        name: "vegetation - medium",
    },
    ClassInfo {
        id: 5,
        name: "vegetation - high",
    },
    ClassInfo {
        id: 6,
        name: "buildings",
    },
    ClassInfo {
        id: 8,
        name: "street furniture",
    },
    ClassInfo {
        id: 11,
        name: "street pavement",
    },
    ClassInfo {
        id: 15,
        name: "cars, trucks",
    },
];

pub fn get_class_name(id: u8) -> String {
    CLASS_MAP
        .iter()
        .find(|c| c.id == id)
        .map_or("unknown", |c| c.name)
        .to_string()
}
