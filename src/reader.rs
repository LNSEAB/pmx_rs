use super::*;
use std::io::Read;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unsupported version")]
    UnsupportedVersion,
    #[error("invalid data: {}", .0)]
    InvalidData(&'static str),
    #[error("io error: {}", .0)]
    Io(std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(src: std::io::Error) -> Self {
        Self::Io(src)
    }
}

pub(crate) struct Reader<T> {
    reader: T,
    encoding: Encoding,
    extended_uv: usize,
    vertex_index: Vec<u8>,
    tex_index: Vec<u8>,
    mat_index: Vec<u8>,
    bone_index: Vec<u8>,
    morph_index: Vec<u8>,
    rig_index: Vec<u8>,
}

impl<T> Reader<T>
where
    T: Read,
{
    pub fn new(reader: T) -> Self {
        Self {
            reader,
            encoding: Encoding::Utf16,
            extended_uv: 0,
            vertex_index: vec![],
            tex_index: vec![],
            mat_index: vec![],
            bone_index: vec![],
            morph_index: vec![],
            rig_index: vec![],
        }
    }

    pub fn read(&mut self) -> Result<Pmx, Error> {
        let header = self.header()?;
        self.encoding = header.encoding;
        self.extended_uv = header.extended_uv as _;
        self.vertex_index = vec![0u8; header.vertex_index_size as usize];
        self.tex_index = vec![0u8; header.texture_index_size as usize];
        self.mat_index = vec![0u8; header.material_index_size as usize];
        self.bone_index = vec![0u8; header.bone_index_size as usize];
        self.morph_index = vec![0u8; header.morph_index_size as usize];
        self.rig_index = vec![0u8; header.rigid_index_size as usize];
        Ok(Pmx {
            header,
            model_info: self.model_info()?,
            vertices: self.vertices()?,
            faces: self.faces()?,
            textures: self.textures()?,
            materials: self.materials()?,
            bones: self.bones()?,
            morphs: self.morphs()?,
            display_groups: self.display_groups()?,
            rigids: self.rigids()?,
            joints: self.joints()?,
        })
    }

    fn read_bin<const N: usize>(&mut self) -> Result<[u8; N], Error> {
        let mut buffer = [0; N];
        self.reader.read_exact(&mut buffer)?;
        Ok(buffer)
    }

    fn read_u8(&mut self) -> Result<u8, Error> {
        Ok(self.read_bin::<1>()?[0])
    }

    fn read_u16(&mut self) -> Result<u16, Error> {
        const SIZE: usize = std::mem::size_of::<u16>();
        Ok(u16::from_le_bytes(self.read_bin::<SIZE>()?))
    }

    fn read_u32(&mut self) -> Result<u32, Error> {
        const SIZE: usize = std::mem::size_of::<u32>();
        Ok(u32::from_le_bytes(self.read_bin::<SIZE>()?))
    }

    fn read_i32(&mut self) -> Result<i32, Error> {
        const SIZE: usize = std::mem::size_of::<i32>();
        Ok(i32::from_le_bytes(self.read_bin::<SIZE>()?))
    }

    fn read_f32(&mut self) -> Result<f32, Error> {
        const SIZE: usize = std::mem::size_of::<f32>();
        Ok(f32::from_le_bytes(self.read_bin::<SIZE>()?))
    }

    fn read_vec<const N: usize>(&mut self) -> Result<[f32; N], Error> {
        let mut buffer = [0.0f32; N];
        for i in 0..N {
            buffer[i] = self.read_f32()?;
        }
        Ok(buffer)
    }

    fn read_vec2(&mut self) -> Result<[f32; 2], Error> {
        self.read_vec::<2>()
    }

    fn read_vec3(&mut self) -> Result<[f32; 3], Error> {
        self.read_vec::<3>()
    }

    fn read_vec4(&mut self) -> Result<[f32; 4], Error> {
        self.read_vec::<4>()
    }

    fn read_string(&mut self) -> Result<String, Error> {
        let len = self.read_u32()? as usize;
        let mut buffer = vec![0u8; len];
        self.reader.read_exact(&mut buffer)?;
        let s = match self.encoding {
            Encoding::Utf16 => unsafe {
                let buffer = std::slice::from_raw_parts(buffer.as_ptr() as *const u16, len / 2);
                String::from_utf16_lossy(&buffer)
            },
            Encoding::Utf8 => String::from_utf8_lossy(&buffer).to_string(),
        };
        Ok(s)
    }

    fn read_signed_index(
        &mut self,
        f: impl FnOnce(&mut Self) -> Result<&Vec<u8>, Error>,
    ) -> Result<Option<usize>, Error> {
        let buffer = f(self)?;
        match buffer.len() {
            1 => {
                let v = i8::from_le_bytes([buffer[0]]);
                Ok((v >= 0).then(|| v as usize))
            }
            2 => {
                let v = i16::from_le_bytes([buffer[0], buffer[1]]);
                Ok((v >= 0).then(|| v as usize))
            }
            4 => {
                let v = i32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
                Ok((v >= 0).then(|| v as usize))
            }
            _ => unreachable!(),
        }
    }

    fn read_vertex_index(&mut self) -> Result<Option<usize>, Error> {
        let buffer = &mut self.vertex_index;
        self.reader.read_exact(buffer)?;
        match buffer.len() {
            1 => {
                let v = u8::from_le_bytes([buffer[0]]);
                Ok(Some(v as usize))
            }
            2 => {
                let v = u16::from_le_bytes([buffer[0], buffer[1]]);
                Ok(Some(v as usize))
            }
            4 => {
                let v = i32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
                Ok((v >= 0).then(|| v as usize))
            }
            _ => unreachable!(),
        }
    }

    fn read_texture_index(&mut self) -> Result<Option<usize>, Error> {
        self.read_signed_index(|this| {
            this.reader.read_exact(&mut this.tex_index)?;
            Ok(&this.tex_index)
        })
    }

    fn read_material_index(&mut self) -> Result<Option<usize>, Error> {
        self.read_signed_index(|this| {
            this.reader.read_exact(&mut this.mat_index)?;
            Ok(&this.mat_index)
        })
    }

    fn read_bone_index(&mut self) -> Result<Option<usize>, Error> {
        self.read_signed_index(|this| {
            this.reader.read_exact(&mut this.bone_index)?;
            Ok(&this.bone_index)
        })
    }

    fn read_morph_index(&mut self) -> Result<Option<usize>, Error> {
        self.read_signed_index(|this| {
            this.reader.read_exact(&mut this.morph_index)?;
            Ok(&this.morph_index)
        })
    }

    fn read_rigid_index(&mut self) -> Result<Option<usize>, Error> {
        self.read_signed_index(|this| {
            this.reader.read_exact(&mut this.rig_index)?;
            Ok(&this.rig_index)
        })
    }

    fn read_index_size(&mut self) -> Result<u8, Error> {
        let v = self.read_u8()?;
        match v {
            1 | 2 | 4 => Ok(v),
            _ => Err(Error::InvalidData("read_index_size")),
        }
    }

    fn header(&mut self) -> Result<Header, Error> {
        let magic = self.read_bin::<4>()?;
        if magic != [b'P', b'M', b'X', b' '] {
            return Err(Error::InvalidData("magic number"));
        }
        let version = self.read_f32()?;
        let bytes = self.read_u8()?;
        if bytes != 8 {
            return Err(Error::InvalidData("header::bytes"));
        }
        let encoding = match self.read_u8()? {
            0 => Encoding::Utf16,
            1 => Encoding::Utf8,
            _ => return Err(Error::InvalidData("header::encoding")),
        };
        Ok(Header {
            version,
            encoding,
            extended_uv: self.read_u8()?,
            vertex_index_size: self.read_index_size()?,
            texture_index_size: self.read_index_size()?,
            material_index_size: self.read_index_size()?,
            bone_index_size: self.read_index_size()?,
            morph_index_size: self.read_index_size()?,
            rigid_index_size: self.read_index_size()?,
        })
    }

    fn model_info(&mut self) -> Result<ModelInfo, Error> {
        Ok(ModelInfo {
            name: self.read_string()?,
            name_en: self.read_string()?,
            comment: self.read_string()?,
            comment_en: self.read_string()?,
        })
    }

    fn vertex(&mut self) -> Result<Vertex, Error> {
        let position = self.read_vec3()?;
        let normal = self.read_vec3()?;
        let uv = self.read_vec2()?;
        let extended_uv = (0..self.extended_uv)
            .map(|_| self.read_vec4())
            .collect::<Result<Vec<_>, Error>>()?;
        let weight = match self.read_u8()? {
            0 => Weight::Bdef1(Bdef1 {
                bone: self.read_bone_index()?,
            }),
            1 => Weight::Bdef2(Bdef2 {
                bones: [self.read_bone_index()?, self.read_bone_index()?],
                weight: self.read_f32()?,
            }),
            2 => Weight::Bdef4(Bdef4 {
                bones: [
                    self.read_bone_index()?,
                    self.read_bone_index()?,
                    self.read_bone_index()?,
                    self.read_bone_index()?,
                ],
                weights: [
                    self.read_f32()?,
                    self.read_f32()?,
                    self.read_f32()?,
                    self.read_f32()?,
                ],
            }),
            3 => Weight::Sdef(Sdef {
                bones: [self.read_bone_index()?, self.read_bone_index()?],
                weight: self.read_f32()?,
                c: self.read_vec3()?,
                r0: self.read_vec3()?,
                r1: self.read_vec3()?,
            }),
            _ => return Err(Error::InvalidData("vertex::weight")),
        };
        let edge_ratio = self.read_f32()?;
        Ok(Vertex {
            position,
            normal,
            uv,
            extended_uv,
            weight,
            edge_ratio,
        })
    }

    fn vertices(&mut self) -> Result<Vec<Vertex>, Error> {
        let len = self.read_u32()?;
        (0..len).map(|_| self.vertex()).collect()
    }

    fn faces(&mut self) -> Result<Vec<u32>, Error> {
        let len = self.read_u32()?;
        (0..len).map(|_| Ok(self.read_u32()?)).collect()
    }

    fn textures(&mut self) -> Result<Vec<PathBuf>, Error> {
        let len = self.read_u32()?;
        (0..len).map(|_| Ok(self.read_string()?.into())).collect()
    }

    fn material(&mut self) -> Result<Material, Error> {
        let name = self.read_string()?;
        let name_en = self.read_string()?;
        let diffuse = self.read_vec4()?;
        let specular = self.read_vec3()?;
        let specular_power = self.read_f32()?;
        let ambient = self.read_vec3()?;
        let flags = self.read_u8()?;
        let both = flags & 0x01 == 0x01;
        let ground_shadow = flags & 0x02 == 0x02;
        let self_shadow_map = flags & 0x04 == 0x04;
        let self_shadow = flags & 0x08 == 0x08;
        let edge = flags & 0x10 == 0x010;
        let edge_color = self.read_vec4()?;
        let edge_size = self.read_f32()?;
        let texture = self.read_texture_index()?;
        let sphere = self.read_texture_index()?;
        let sphere_mode = match self.read_u8()? {
            0 => SphereMode::None,
            1 => SphereMode::Mul,
            2 => SphereMode::Add,
            3 => SphereMode::SubTexture,
            _ => return Err(Error::InvalidData("material::sphere_mode")),
        };
        let toon = match self.read_u8()? {
            0 => Toon::Texture(self.read_texture_index()?),
            1 => Toon::Shared(self.read_u8()? as _),
            _ => return Err(Error::InvalidData("material::toon")),
        };
        let memo = self.read_string()?;
        let index_count = self.read_u32()?;
        if index_count % 3 != 0 {
            return Err(Error::InvalidData("material::index_count"));
        }
        Ok(Material {
            name,
            name_en,
            diffuse,
            specular,
            specular_power,
            ambient,
            both,
            ground_shadow,
            self_shadow_map,
            self_shadow,
            edge,
            edge_color,
            edge_size,
            texture,
            sphere,
            sphere_mode,
            toon,
            memo,
            index_count,
        })
    }

    fn materials(&mut self) -> Result<Vec<Material>, Error> {
        let len = self.read_u32()?;
        (0..len).map(|_| self.material()).collect()
    }

    fn bone(&mut self) -> Result<Bone, Error> {
        let name = self.read_string()?;
        let name_en = self.read_string()?;
        let position = self.read_vec3()?;
        let parent = self.read_bone_index()?;
        let deform_hierarchy = self.read_i32()?;
        let flags = self.read_u16()?;
        let connected_to = match flags & 0x0001 {
            0 => ConnectedTo::Offset(self.read_vec3()?),
            1 => ConnectedTo::Bone(self.read_bone_index()?),
            _ => return Err(Error::InvalidData("bone::connected_to")),
        };
        let rotatable = flags & 0x0002 == 0x0002;
        let translatable = flags & 0x0004 == 0x0004;
        let visibility = flags & 0x0008 == 0x0008;
        let operable = flags & 0x0010 == 0x0010;
        let addition = {
            let rotation = flags & 0x0100 == 0x0100;
            let translation = flags & 0x0200 == 0x0200;
            if !(rotation || translation) {
                None
            } else {
                let local = flags & 0x0080 == 0x0080;
                let bone = self.read_bone_index()?;
                let ratio = self.read_f32()?;
                Some(Addition {
                    rotation,
                    translation,
                    local,
                    bone,
                    ratio,
                })
            }
        };
        let fixed_pole = (flags & 0x0400 == 0x0400)
            .then(|| self.read_vec3())
            .transpose()?;
        let local_pole = (flags & 0x0800 == 0x0800)
            .then(|| {
                Ok::<_, Error>(LocalPole {
                    x: self.read_vec3()?,
                    z: self.read_vec3()?,
                })
            })
            .transpose()?;
        let after_physics = flags & 0x1000 == 0x1000;
        let external_parent = (flags & 0x2000 == 0x2000)
            .then(|| {
                let v = self.read_i32()?;
                Ok::<_, Error>((v >= 0).then(|| v as usize))
            })
            .transpose()?
            .flatten();
        let ik = (flags & 0x0020 == 0x0020)
            .then(|| {
                let bone = self.read_bone_index()?;
                let loop_count = self.read_u32()?;
                let angle = self.read_f32()?;
                let link_len = self.read_u32()?;
                let links = (0..link_len)
                    .map(|_| {
                        let bone = self.read_bone_index()?;
                        let limits = (self.read_u8()? == 0x01)
                            .then(|| {
                                Ok::<_, Error>(AngleLimit {
                                    lower: self.read_vec3()?,
                                    upper: self.read_vec3()?,
                                })
                            })
                            .transpose()?;
                        Ok(IkLink { bone, limits })
                    })
                    .collect::<Result<Vec<_>, Error>>()?;
                Ok::<_, Error>(Ik {
                    bone,
                    loop_count,
                    angle,
                    links,
                })
            })
            .transpose()?;
        Ok(Bone {
            name,
            name_en,
            position,
            parent,
            deform_hierarchy,
            connected_to,
            rotatable,
            translatable,
            visibility,
            operable,
            ik,
            addition,
            after_physics,
            fixed_pole,
            local_pole,
            external_parent,
        })
    }

    fn bones(&mut self) -> Result<Vec<Bone>, Error> {
        let len = self.read_u32()?;
        (0..len).map(|_| self.bone()).collect()
    }

    fn morph(&mut self) -> Result<Morph, Error> {
        let name = self.read_string()?;
        let name_en = self.read_string()?;
        let panel = match self.read_u8()? {
            0 => Panel::Reserved,
            1 => Panel::Eyebrow,
            2 => Panel::Eye,
            3 => Panel::Mouth,
            4 => Panel::Other,
            _ => return Err(Error::InvalidData("morph::panel")),
        };
        let kind_value = self.read_u8()?;
        let len = self.read_u32()?;
        let kind = match kind_value {
            0 => morph::Kind::Group(
                (0..len)
                    .map(|_| {
                        Ok(morph::Group {
                            morph: self.read_morph_index()?,
                            ratio: self.read_f32()?,
                        })
                    })
                    .collect::<Result<_, Error>>()?,
            ),
            1 => morph::Kind::Vertex(
                (0..len)
                    .map(|_| {
                        Ok(morph::Vertex {
                            vertex: self.read_vertex_index()?,
                            offset: self.read_vec3()?,
                        })
                    })
                    .collect::<Result<_, Error>>()?,
            ),
            2 => morph::Kind::Bone(
                (0..len)
                    .map(|_| {
                        Ok(morph::Bone {
                            bone: self.read_bone_index()?,
                            offset: self.read_vec3()?,
                            rotation: self.read_vec4()?,
                        })
                    })
                    .collect::<Result<_, Error>>()?,
            ),
            3 => morph::Kind::Uv(
                (0..len)
                    .map(|_| {
                        Ok(morph::Uv {
                            vertex: self.read_vertex_index()?,
                            offset: self.read_vec4()?,
                        })
                    })
                    .collect::<Result<_, Error>>()?,
            ),
            i @ 4..=7 => morph::Kind::ExtendedUv(
                (i - 4) as _,
                (0..len)
                    .map(|_| {
                        Ok(morph::Uv {
                            vertex: self.read_vertex_index()?,
                            offset: self.read_vec4()?,
                        })
                    })
                    .collect::<Result<_, Error>>()?,
            ),
            8 => morph::Kind::Maerial(
                (0..len)
                    .map(|_| {
                        Ok(morph::Material {
                            material: self.read_material_index()?,
                            op: match self.read_u8()? {
                                0 => morph::MaterialOp::Mul,
                                1 => morph::MaterialOp::Add,
                                _ => return Err(Error::InvalidData("morph::Material::op")),
                            },
                            diffuse: self.read_vec4()?,
                            specular: self.read_vec3()?,
                            specular_power: self.read_f32()?,
                            ambient: self.read_vec3()?,
                            edge_color: self.read_vec4()?,
                            edge_size: self.read_f32()?,
                            texture: self.read_vec4()?,
                            sphere: self.read_vec4()?,
                            toon: self.read_vec4()?,
                        })
                    })
                    .collect::<Result<_, Error>>()?,
            ),
            _ => return Err(Error::InvalidData("morph::kind")),
        };
        Ok(Morph {
            name,
            name_en,
            panel,
            kind,
        })
    }

    fn morphs(&mut self) -> Result<Vec<Morph>, Error> {
        let len = self.read_u32()?;
        (0..len).map(|_| self.morph()).collect()
    }

    fn display_group(&mut self) -> Result<DisplayGroup, Error> {
        let name = self.read_string()?;
        let name_en = self.read_string()?;
        let special = self.read_u8()? == 1;
        let len = self.read_u32()?;
        let elements = (0..len)
            .map(|_| {
                let t = self.read_u8()?;
                Ok(match t {
                    0 => DisplayElement::Bone(self.read_bone_index()?),
                    1 => DisplayElement::Morph(self.read_morph_index()?),
                    _ => return Err(Error::InvalidData("display_group::elements")),
                })
            })
            .collect::<Result<_, Error>>()?;
        Ok(DisplayGroup {
            name,
            name_en,
            special,
            elements,
        })
    }

    fn display_groups(&mut self) -> Result<Vec<DisplayGroup>, Error> {
        let len = self.read_u32()?;
        (0..len).map(|_| self.display_group()).collect()
    }

    fn rigid(&mut self) -> Result<Rigid, Error> {
        let name = self.read_string()?;
        let name_en = self.read_string()?;
        let bone = self.read_bone_index()?;
        let group = self.read_u8()?;
        let non_collision_groups = self.read_u16()?;
        let shape = match self.read_u8()? {
            0 => rigid::Shape::Sphere,
            1 => rigid::Shape::Box,
            2 => rigid::Shape::Capsule,
            _ => return Err(Error::InvalidData("rigid::shape")),
        };
        let size = self.read_vec3()?;
        let position = self.read_vec3()?;
        let rotation = self.read_vec3()?;
        let mass = self.read_f32()?;
        let dump_translation = self.read_f32()?;
        let dump_rotation = self.read_f32()?;
        let repulsive = self.read_f32()?;
        let friction = self.read_f32()?;
        let method = match self.read_u8()? {
            0 => rigid::Method::Static,
            1 => rigid::Method::Dynamic,
            2 => rigid::Method::DynamicWithBone,
            _ => return Err(Error::InvalidData("rigid::method")),
        };
        Ok(Rigid {
            name,
            name_en,
            bone,
            group,
            non_collision_groups,
            shape,
            size,
            position,
            rotation,
            mass,
            dump_translation,
            dump_rotation,
            repulsive,
            friction,
            method,
        })
    }

    fn rigids(&mut self) -> Result<Vec<Rigid>, Error> {
        let len = self.read_u32()?;
        (0..len).map(|_| self.rigid()).collect()
    }

    fn joint(&mut self) -> Result<Joint, Error> {
        let name = self.read_string()?;
        let name_en = self.read_string()?;
        let t = self.read_u8()?;
        if t != 0 {
            return Err(Error::InvalidData("joint::type"));
        }
        let rigids = [self.read_rigid_index()?, self.read_rigid_index()?];
        let position = self.read_vec3()?;
        let rotation = self.read_vec3()?;
        let limit_translation = AngleLimit {
            lower: self.read_vec3()?,
            upper: self.read_vec3()?,
        };
        let limit_rotation = AngleLimit {
            lower: self.read_vec3()?,
            upper: self.read_vec3()?,
        };
        let spring_translation = self.read_vec3()?;
        let spring_rotation = self.read_vec3()?;
        Ok(Joint {
            name,
            name_en,
            rigids,
            position,
            rotation,
            limit_translation,
            limit_rotation,
            spring_translation,
            spring_rotation,
        })
    }

    fn joints(&mut self) -> Result<Vec<Joint>, Error> {
        let len = self.read_u32()?;
        (0..len).map(|_| self.joint()).collect()
    }
}
