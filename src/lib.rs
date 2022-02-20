mod reader;

use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(u8)]
pub enum Encoding {
    Utf16 = 0,
    Utf8 = 1,
}

#[derive(Clone, Debug)]
pub struct Header {
    pub version: f32,
    pub encoding: Encoding,
    pub extended_uv: u8,
    pub vertex_index_size: u8,
    pub texture_index_size: u8,
    pub material_index_size: u8,
    pub bone_index_size: u8,
    pub morph_index_size: u8,
    pub rigid_index_size: u8,
}

#[derive(Clone, Debug)]
pub struct ModelInfo {
    pub name: String,
    pub name_en: String,
    pub comment: String,
    pub comment_en: String,
}

#[derive(Clone, Debug)]
pub struct Bdef1 {
    pub bone: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct Bdef2 {
    pub bones: [Option<usize>; 2],
    pub weight: f32,
}

#[derive(Clone, Debug)]
pub struct Bdef4 {
    pub bones: [Option<usize>; 4],
    pub weights: [f32; 4],
}

#[derive(Clone, Debug)]
pub struct Sdef {
    pub bones: [Option<usize>; 2],
    pub weight: f32,
    pub c: [f32; 3],
    pub r0: [f32; 3],
    pub r1: [f32; 3],
}

#[derive(Clone, Debug)]
pub enum Weight {
    Bdef1(Bdef1),
    Bdef2(Bdef2),
    Bdef4(Bdef4),
    Sdef(Sdef),
}

#[derive(Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub extended_uv: Vec<[f32; 4]>,
    pub weight: Weight,
    pub edge_ratio: f32,
}

#[derive(Clone, Debug)]
pub enum SphereMode {
    None,
    Mul,
    Add,
    SubTexture,
}

#[derive(Clone, Debug)]
pub enum Toon {
    Texture(Option<usize>),
    Shared(u32),
}

#[derive(Clone, Debug)]
pub struct Material {
    pub name: String,
    pub name_en: String,
    pub diffuse: [f32; 4],
    pub specular: [f32; 3],
    pub specular_power: f32,
    pub ambient: [f32; 3],
    pub both: bool,
    pub ground_shadow: bool,
    pub self_shadow_map: bool,
    pub self_shadow: bool,
    pub edge: bool,
    pub edge_color: [f32; 4],
    pub edge_size: f32,
    pub texture: Option<usize>,
    pub sphere: Option<usize>,
    pub sphere_mode: SphereMode,
    pub toon: Toon,
    pub memo: String,
    pub index_count: u32,
}

#[derive(Clone, Debug)]
pub enum ConnectedTo {
    Offset([f32; 3]),
    Bone(Option<usize>),
}

#[derive(Clone, Debug)]
pub struct AngleLimit {
    pub lower: [f32; 3],
    pub upper: [f32; 3],
}

#[derive(Clone, Debug)]
pub struct IkLink {
    pub bone: Option<usize>,
    pub limits: Option<AngleLimit>,
}

#[derive(Clone, Debug)]
pub struct Ik {
    pub bone: Option<usize>,
    pub loop_count: u32,
    pub angle: f32,
    pub links: Vec<IkLink>,
}

#[derive(Clone, Debug)]
pub struct Addition {
    pub rotation: bool,
    pub translation: bool,
    pub local: bool,
    pub bone: Option<usize>,
    pub ratio: f32,
}

#[derive(Clone, Debug)]
pub struct LocalPole {
    pub x: [f32; 3],
    pub z: [f32; 3],
}

#[derive(Clone, Debug)]
pub struct Bone {
    pub name: String,
    pub name_en: String,
    pub position: [f32; 3],
    pub parent: Option<usize>,
    pub deform_hierarchy: i32,
    pub connected_to: ConnectedTo,
    pub rotatable: bool,
    pub translatable: bool,
    pub visibility: bool,
    pub operable: bool,
    pub ik: Option<Ik>,
    pub addition: Option<Addition>,
    pub after_physics: bool,
    pub fixed_pole: Option<[f32; 3]>,
    pub local_pole: Option<LocalPole>,
    pub external_parent: Option<usize>,
}

#[derive(Clone, Debug)]
pub enum Panel {
    Reserved,
    Eyebrow,
    Eye,
    Mouth,
    Other,
}

pub mod morph {
    #[derive(Clone, Debug)]
    pub struct Vertex {
        pub vertex: Option<usize>,
        pub offset: [f32; 3],
    }

    #[derive(Clone, Debug)]
    pub struct Uv {
        pub vertex: Option<usize>,
        pub offset: [f32; 4],
    }

    #[derive(Clone, Debug)]
    pub struct Bone {
        pub bone: Option<usize>,
        pub offset: [f32; 3],
        pub rotation: [f32; 4],
    }

    #[derive(Clone, PartialEq, Eq, Debug)]
    pub enum MaterialOp {
        Mul,
        Add,
    }

    #[derive(Clone, Debug)]
    pub struct Material {
        pub material: Option<usize>,
        pub op: MaterialOp,
        pub diffuse: [f32; 4],
        pub specular: [f32; 3],
        pub specular_power: f32,
        pub ambient: [f32; 3],
        pub edge_color: [f32; 4],
        pub edge_size: f32,
        pub texture: [f32; 4],
        pub sphere: [f32; 4],
        pub toon: [f32; 4],
    }

    #[derive(Clone, Debug)]
    pub struct Group {
        pub morph: Option<usize>,
        pub ratio: f32,
    }

    #[derive(Clone, Debug)]
    pub enum Kind {
        Vertex(Vec<Vertex>),
        Uv(Vec<Uv>),
        Bone(Vec<Bone>),
        Maerial(Vec<Material>),
        Group(Vec<Group>),
        ExtendedUv(usize, Vec<Uv>),
    }
}

#[derive(Clone, Debug)]
pub struct Morph {
    pub name: String,
    pub name_en: String,
    pub panel: Panel,
    pub kind: morph::Kind,
}

#[derive(Clone, Debug)]
pub enum DisplayElement {
    Bone(Option<usize>),
    Morph(Option<usize>),
}

#[derive(Clone, Debug)]
pub struct DisplayGroup {
    pub name: String,
    pub name_en: String,
    pub special: bool,
    pub elements: Vec<DisplayElement>,
}

pub mod rigid {
    #[derive(Clone, Debug)]
    pub enum Shape {
        Sphere,
        Box,
        Capsule,
    }

    #[derive(Clone, Debug)]
    pub enum Method {
        Static,
        Dynamic,
        DynamicWithBone,
    }
}

#[derive(Clone, Debug)]
pub struct Rigid {
    pub name: String,
    pub name_en: String,
    pub bone: Option<usize>,
    pub group: u8,
    pub non_collision_groups: u16,
    pub shape: rigid::Shape,
    pub size: [f32; 3],
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub mass: f32,
    pub dump_translation: f32,
    pub dump_rotation: f32,
    pub repulsive: f32,
    pub friction: f32,
    pub method: rigid::Method,
}

#[derive(Clone, Debug)]
pub struct Joint {
    pub name: String,
    pub name_en: String,
    pub rigids: [Option<usize>; 2],
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub limit_translation: AngleLimit,
    pub limit_rotation: AngleLimit,
    pub spring_translation: [f32; 3],
    pub spring_rotation: [f32; 3],
}

#[derive(Clone, Debug)]
pub struct Pmx {
    pub header: Header,
    pub model_info: ModelInfo,
    pub vertices: Vec<Vertex>,
    pub faces: Vec<u32>,
    pub textures: Vec<PathBuf>,
    pub materials: Vec<Material>,
    pub bones: Vec<Bone>,
    pub morphs: Vec<Morph>,
    pub display_groups: Vec<DisplayGroup>,
    pub rigids: Vec<Rigid>,
    pub joints: Vec<Joint>,
}

#[inline]
pub fn read<T: std::io::Read>(reader: T) -> Result<Pmx, reader::Error> {
    let mut reader = reader::Reader::new(reader);
    reader.read()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_pmx() -> Pmx {
        let file = std::fs::File::open("resource/Alicia/Alicia_solid.pmx").unwrap();
        let reader = std::io::BufReader::new(file);
        read(reader).unwrap()
    }

    #[test]
    fn model_name() {
        let pmx = read_pmx();
        assert!(pmx.model_info.name == "アリシア・ソリッド");
    }

    #[test]
    fn vertex_size() {
        let pmx = read_pmx();
        assert!(pmx.vertices.len() == 22311);
    }

    #[test]
    fn face_size() {
        let pmx = read_pmx();
        assert!(pmx.faces.len() == 95598);
    }

    #[test]
    fn material_size() {
        let pmx = read_pmx();
        assert!(pmx.materials.len() == 22);
    }

    #[test]
    fn bone_size() {
        let pmx = read_pmx();
        assert!(pmx.bones.len() == 150);
    }

    #[test]
    fn rigid_size() {
        let pmx = read_pmx();
        assert!(pmx.rigids.len() == 79);
    }

    #[test]
    fn joint_size() {
        let pmx = read_pmx();
        assert!(pmx.joints.len() == 53);
    }

    #[test]
    fn last_joint() {
        let pmx = read_pmx();
        assert!(pmx.joints[52].name == "リボン右");
    }
}