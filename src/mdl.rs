// https://github.com/Sevii77/aetherment/blob/master/noumenon/src/format/game/mdl.rs
// heavily trimmed down version to just get what we need to manually edit the byte array
// i havent implemented a writer yet and for this purpose of just adjusting uv's doign this is simple enough
// tried using physis one and it producded broken model files with 0 changes, and didnt even seem possible to change the mtrl paths

macro_rules! simple_reader {
	($y:expr, $z:expr) => {
		let reader = $y;
		let endian = $z;
		
		macro_rules! r {
			(move $c:expr) => {{
				reader.seek_relative($c as i64)?
			}};
			
			(seek $c:expr) => {{
				reader.seek(::std::io::SeekFrom::Start($c as u64))?
			}};
			
			(eof) => {{
				let mut v = Vec::new();
				reader.read_to_end(&mut v)?;
				v
			}};
			
			(Vec<$e:ty>, $c:expr) => {{
				let mut v = Vec::with_capacity($c as usize);
				for _ in 0..$c {
					v.push(<$e>::read_options(reader, endian, ())?);
				}
				v
			}};
			
			(f16) => {{
				half::f16::from_bits(r!(u16)).to_f32()
			}};
			
			($e:ty) => {{
				<$e>::read_options(reader, endian, ())?
			}};
			
			($f:ident, $a:tt) => {{
				$f(reader, endian, $a)?
			}};
		}
	};
}

pub trait NullReader {
	fn null_terminated(&self) -> Result<String, std::str::Utf8Error>;
}

impl NullReader for [u8] {
	fn null_terminated(&self) -> Result<String, std::str::Utf8Error> {
		let p = std::str::from_utf8(&self)?;
		Ok(if let Some(l) = p.find('\0') {&p[0..l]} else {p}.to_owned())
	}
}


use std::{fmt::Debug, io::{Read, Seek, SeekFrom}};
use binrw::{binrw, BinRead};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Mdl {
	pub strings_buf: Vec<u8>,
	pub header: HeaderRaw,
	pub vertex_declerations: Vec<[VertexElementRaw; 17]>,
	pub lods: Vec<LodRaw>,
	pub meshes: Vec<MeshRaw>,
	pub submeshes: Vec<SubmeshRaw>,
	pub material_string_offset: Vec<u32>,
}

impl BinRead for Mdl {
	type Args<'a> = ();
	
	fn read_options<R: Read + Seek>(mut reader: &mut R, endian: binrw::Endian, _args: Self::Args<'_>,) -> binrw::BinResult<Self> {
		simple_reader!(&mut reader, endian);
		
		let header = r!(HeaderRaw);
		let vertex_declerations = r!(Vec<[VertexElementRaw; 17]>, header.vertex_declaration_count);
		
		let _strings_count = r!(u16);
		r!(move 2);
		let strings_size = r!(u32);
		let strings_buf = r!(Vec<u8>, strings_size);
		
		let model_header = r!(ModelHeaderRaw);
		let _element_ids = r!(Vec<ElementIdRaw>, model_header.element_id_count);
		let lods = r!(Vec<LodRaw>, 3);
		let _extra_lods = r!(Vec<ExtraLodRaw>, if model_header.flags2.contains(ModelFlags2Raw::EXTRA_LOD_ENABLED) {3} else {0});
		let meshes = r!(Vec<MeshRaw>, model_header.mesh_count);
		let _attribute_string_offset = r!(Vec<u32>, model_header.attribute_count);
		let _terrain_shadow_meshes = r!(Vec<TerrainShadowMeshRaw>, model_header.terrain_shadow_mesh_count);
		let submeshes = r!(Vec<SubmeshRaw>, model_header.submesh_count);
		let _terrain_shadow_submeshes = r!(Vec<TerrainShadowSubmeshRaw>, model_header.terrain_shadow_submesh_count);
		let material_string_offset = r!(Vec<u32>, model_header.material_count);
		let _bone_string_offset = r!(Vec<u32>, model_header.bone_count);
		let _bone_table = r!(bone_table_reader, (header.version, model_header.bone_table_count, model_header.bone_table_array_count_total));
		let _shapes = r!(Vec<ShapeRaw>, model_header.shape_count);
		let _shape_meshes = r!(Vec<ShapeMeshRaw>, model_header.shape_mesh_count);
		let _shape_values = r!(Vec<ShapeValueRaw>, model_header.shape_value_count);
		let submesh_bone_map_size = r!(u32);
		let _submesh_bone_map = r!(Vec<u16>, submesh_bone_map_size / 2);
		let _neck_morth = r!(Vec<NeckMorphRaw>, model_header.neck_morph_count);
		let _unkown_face_shadow_data = r!(Vec<UnkFaceShadowDataRaw>, model_header.unknown_face_shadow_data_count);
		
		let padding_size = r!(u8);
		r!(move padding_size);
		
		let _bb = r!(BoundingBoxRaw);
		let _model_bb = r!(BoundingBoxRaw);
		let _water_bb = r!(BoundingBoxRaw);
		let _vertical_fog_bb = r!(BoundingBoxRaw);
		let _bones_bb = r!(Vec<BoundingBoxRaw>, model_header.bone_count);
		
		Ok(Self {
			strings_buf,
			header,
			vertex_declerations,
			lods,
			meshes,
			submeshes,
			material_string_offset,
		})
	}
}

// ----------

#[binrw]
#[derive(Debug, Clone)]
pub struct HeaderRaw {
	pub version: u32,
	pub stack_size: u32,
	pub runtime_size: u32,
	pub vertex_declaration_count: u16,
	pub material_count: u16,
	pub vertex_offsets: [u32; 3],
	pub index_offsets: [u32; 3],
	pub vertex_buffer_offsets: [u32; 3],
	pub index_buffer_offsets: [u32; 3],
	pub lod_count: u8,
	pub index_buffer_streaming: u8,
	pub edge_geometry: u8,
	pub _padding: u8,
}

#[binrw]
#[derive(Debug, Clone)]
pub struct VertexElementRaw {
	pub stream: u8,
	pub offset: u8,
	pub typ: VertexTypeRaw,
	pub usage: VertexUsageRaw,
	pub usage_index: u8,
	pub _padding: [u8; 3],
}

#[binrw]
#[brw(repr = u8)]
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum VertexTypeRaw {
	F32x1 = 0,
	F32x2 = 1,
	F32x3 = 2,
	F32x4 = 3,
	U8x4  = 5,
	F8x4  = 8,
	F16x2 = 13,
	F16x4 = 14,
	U16x2 = 16,
	U16x4 = 17,
}

#[binrw]
#[brw(repr = u8)]
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum VertexUsageRaw {
	Position     = 0,
	BlendWeights = 1,
	BlendIndices = 2,
	Normal       = 3,
	Uv           = 4,
	Tangent2     = 5,
	Tangent1     = 6,
	Color        = 7,
}

#[binrw]
#[derive(Debug, Clone)]
struct ModelHeaderRaw {
	radius: f32,
	mesh_count: u16,
	attribute_count: u16,
	submesh_count: u16,
	material_count: u16,
	bone_count: u16,
	bone_table_count: u16,
	shape_count: u16,
	shape_mesh_count: u16,
	shape_value_count: u16,
	lod_count: u8,
	#[br(map = |v: u8| ModelFlags1Raw::from_bits_retain(v))]
	#[bw(map = |v: &ModelFlags1Raw| v.bits())]
	flags1: ModelFlags1Raw,
	element_id_count: u16,
	terrain_shadow_mesh_count: u8,
	#[br(map = |v: u8| ModelFlags2Raw::from_bits_retain(v))]
	#[bw(map = |v: &ModelFlags2Raw| v.bits())]
	flags2: ModelFlags2Raw,
	model_clip_out_distance: f32,
	shadow_clip_out_distance: f32,
	culling_grid_count: u16,
	terrain_shadow_submesh_count: u16,
	flags3: u8, // ?
	bg_change_material_index: u8,
	bg_crest_change_material_index: u8,
	neck_morph_count: u8,
	bone_table_array_count_total: u16,
	unknown8: u16,
	unknown_face_shadow_data_count: u16,
	unknown9: u16,
	unknown10: u16,
	unknown11: u16,
}

bitflags::bitflags! {
	#[repr(transparent)]
	#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
	struct ModelFlags1Raw: u8 {
		const DUST_OCCLUSION_ENABLED = 0x80;
		const SNOW_OCCLUSION_ENABLED = 0x40;
		const RAIN_OCCLUSION_ENABLED = 0x20;
		const UNKNOWN1 = 0x10;
		const LIGHTING_REFLECTION_ENABLED = 0x08;
		const WAVING_ANIMATION_DISABLED = 0x04;
		const LIGHT_SHADOW_DISABLED = 0x02;
		const SHADOW_DISABLED = 0x01;
	}
}

bitflags::bitflags! {
	#[repr(transparent)]
	#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
	struct ModelFlags2Raw: u8 {
		const UNKNOWN2 = 0x80;
		const BG_UV_SCROLL_ENABLED = 0x40;
		const FORCE_NON_RESIDENT_ENABLED = 0x20;
		const EXTRA_LOD_ENABLED = 0x10;
		const SHADOW_MASK_ENABLED = 0x08;
		const FORCE_LOD_RANGE_ENABLED = 0x04;
		const EDGE_GEOMETRY_ENABLED = 0x02;
		const UNKINWO3 = 0x0;
	}
}

#[binrw]
#[derive(Debug, Clone)]
struct ElementIdRaw {
	element_id: u32,
	parent_bone_name: u32,
	translation: [f32; 3],
	rotation: [f32; 3],
}

#[binrw]
#[derive(Debug, Clone)]
pub struct LodRaw {
	pub mesh_index: u16,
	pub mesh_count: u16,
	pub model_lod_range: f32,
	pub texture_load_range: f32,
	pub water_mesh_index: u16,
	pub water_mesh_count: u16,
	pub shadow_mesh_index: u16,
	pub shadow_mesh_count: u16,
	pub terrain_shadow_mesh_index: u16,
	pub terrain_shadow_mesh_count: u16,
	pub vertical_fog_mesh_index: u16,
	pub vertical_fog_mesh_count: u16,
	
	pub edge_geometry_size: u32,
	pub edge_geometry_data_offset: u32,
	pub polygon_count: u32,
	pub unknown1: u32,
	pub vertex_buffer_size: u32,
	pub index_buffer_size: u32,
	pub vertex_data_offset: u32,
	pub index_data_offset: u32,
}

#[binrw]
#[derive(Debug, Clone)]
struct ExtraLodRaw {
	lightshaft_mesh_index: u16,
	lightshaft_mesh_count: u16,
	glass_mesh_index: u16,
	glass_mesh_count: u16,
	material_change_mesh_index: u16,
	material_change_mesh_count: u16,
	crest_change_mesh_index: u16,
	crest_change_mesh_count: u16,
	unknown1: u16,
	unknown2: u16,
	unknown3: u16,
	unknown4: u16,
	unknown5: u16,
	unknown6: u16,
	unknown7: u16,
	unknown8: u16,
	unknown9: u16,
	unknown10: u16,
	unknown11: u16,
	unknown12: u16,
}

#[binrw]
#[derive(Debug, Clone)]
pub struct MeshRaw {
	pub vertex_count: u16,
	pub _padding: u16,
	pub index_count: u32,
	pub material_index: u16,
	pub submesh_index: u16,
	pub submesh_count: u16,
	pub bone_table_index: u16,
	pub start_index: u32,
	pub vertex_buffer_offset: [u32; 3],
	pub vertex_buffer_stride: [u8; 3],
	pub vertex_stream_count: u8,
}

#[binrw]
#[derive(Debug, Clone)]
struct TerrainShadowMeshRaw {
	index_count: u32,
	start_index: u32,
	vertex_buffer_offset: u32,
	vertex_count: u16,
	submesh_index: u16,
	submesh_count: u16,
	vertex_buffer_stride: u8,
	_padding: u8
}

#[binrw]
#[derive(Debug, Clone)]
pub struct SubmeshRaw {
	pub index_offset: u32,
	pub index_count: u32,
	pub attribute_index_mask: u32,
	pub bone_start_index: u16,
	pub bone_count: u16,
}

#[binrw]
#[derive(Debug, Clone)]
struct TerrainShadowSubmeshRaw {
	index_offset: u32,
	index_count: u32,
	unknown1: u16,
	unknown2: u16,
}

#[binrw::parser(reader, endian)]
fn bone_table_reader(version: u32, count: u16, count_total: u16) -> binrw::BinResult<Vec<Vec<u16>>> {
	match version & 0xFF {
		5 => {
			let mut bones_all = Vec::with_capacity(count as usize);
			for _ in 0..count {
				let bones = binrw::BinReaderExt::read_type::<[u16; 64]>(reader, endian)?;
				let count = u32::read_options(reader, endian, ())?;
				bones_all.push(bones[..count as usize].to_vec())
			}
			
			Ok(bones_all)
		}
		
		6 => {
			let mut bones_all = Vec::with_capacity(count as usize);
			for _ in 0..count {
				let pos = reader.stream_position()?;
				let offset = u16::read_options(reader, endian, ())?;
				let count = u16::read_options(reader, endian, ())?;
				reader.seek(SeekFrom::Start(pos + offset as u64 * 4))?;
				let mut bones = vec![0u16; count as usize];
				for i in 0..count {
					bones[i as usize] = u16::read_options(reader, endian, ())?;
				}
				reader.seek(SeekFrom::Start(pos + 4))?;
				bones_all.push(bones);
			}
			
			reader.seek(SeekFrom::Current(count_total as i64 * 2))?;
			
			Ok(bones_all)
		}
		
		_ => {
			Err(binrw::Error::BadMagic{pos: 0, found: Box::new(version & 0xFF)})
		}
	}
}

#[binrw]
#[derive(Debug, Clone)]
struct ShapeRaw {
	string_offset: u32,
	mesh_start_index: [u16; 3],
	mesh_count: [u16; 3],
}

#[binrw]
#[derive(Debug, Clone)]
struct ShapeMeshRaw {
	mesh_index_offset: u32,
	value_count: u32,
	value_offset: u32,
}

#[binrw]
#[derive(Debug, Clone)]
struct ShapeValueRaw {
	base_indices_index: u16,
	replacing_vertex_index: u16,
}

#[binrw]
#[derive(Debug, Clone)]
struct NeckMorphRaw {
	rel_position: [f32; 3],
	unknown1: [u8; 4],
	rel_normal: [f32; 3],
	bone_table: [u8; 4],
}

#[binrw]
#[derive(Debug, Clone)]
struct UnkFaceShadowDataRaw {
	unknown1: f32,
	unknown2: f32,
	unknown3: f32,
	unknown4: u32,
}

#[binrw]
#[derive(Debug, Clone)]
struct BoundingBoxRaw {
	min: [f32; 4],
	max: [f32; 4],
}