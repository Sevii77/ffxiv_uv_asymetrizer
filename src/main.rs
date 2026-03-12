use std::{fs::File, io::{Seek, Write}};
use binrw::BinRead;
use mdl::NullReader;
use physis::resource::Resource;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

mod mdl;

const DESC: &str = "\
Every single vanilla gear piece, including body, legs, hands, and feet, with an asymmetrical body texture.
Enjoy your tattoos and other without having to go searching for the specific upscale for some random gearpiece.

Automatically converted with https://github.com/Sevii77/ffxiv_uv_asymetrizer

Material files:
\tBibo: _q.mtrl
\tTBSE: _b.mtrl
\tHR3: _b.mtrl

Q: Why does bibo not use _bibo.mtrl?
A: The way I modify the mdl files doesn't allow me to make any changes to size, I simply provide the mtrl files from my install.
\tThis shouldn't cause any issue since the textures are all the same, but if it does let me know.
\tI tried using file swaps to swap _q for _bibo, but that didn't seem to work.

Q: Why does the Bibo/HR3 and TBSE versions have conflicts?
A: If a non male hyur body doesn't have a specific model, it defaults to that.
\tThis could be fixed with EQDP edits, but sadly that also requires realigning and scaling the mesh,
\tI might look into that in the future.";

fn main() {
	let mut args = std::env::args();
	args.next();
	
	let (Some(version), Some(game_dir)) = (args.next(), args.next()) else {
		println!("Usage: ffxiv_uv_asymetrizer [version] [game_dir]");
		return;
	};
	
	_ = std::fs::remove_dir_all("./mod");
	
	// otopop actually isnt vanilla but twice
	for body_type in [BodyType::TallFemale, BodyType::TallMale, BodyType::BigMale] {
	// for body_type in [BodyType::TallFemale, BodyType::TallMale, BodyType::BigMale, BodyType::Lalafell] {
		let body_name = body_type.name();
		let mod_name = format!("Asymmetrical Vanilla Gear ({body_name})");
		let mod_path = format!("mod/{mod_name}");
		_ = std::fs::create_dir_all(format!("{mod_path}/files"));
		
		let mut files;
		if body_type == BodyType::TallFemale {
			files = vec![
				("chara/human/c0201/obj/body/b0001/material/v0001/mt_c0201b0001_q.mtrl".to_string(), "files/bibomtrl/c0201b0001.mtrl".to_string()),
				("chara/human/c0401/obj/body/b0001/material/v0001/mt_c0401b0001_q.mtrl".to_string(), "files/bibomtrl/c0401b0001.mtrl".to_string()),
				("chara/human/c1401/obj/body/b0001/material/v0001/mt_c1401b0001_q.mtrl".to_string(), "files/bibomtrl/c1401b0001.mtrl".to_string()),
				("chara/human/c1401/obj/body/b0101/material/v0001/mt_c1401b0101_q.mtrl".to_string(), "files/bibomtrl/c1401b0101.mtrl".to_string()),
				("chara/human/c1601/obj/body/b0001/material/v0001/mt_c1601b0001_q.mtrl".to_string(), "files/bibomtrl/c1601b0001_v1.mtrl".to_string()),
				("chara/human/c1601/obj/body/b0001/material/v0002/mt_c1601b0001_q.mtrl".to_string(), "files/bibomtrl/c1601b0001_v2.mtrl".to_string()),
				("chara/human/c1601/obj/body/b0001/material/v0003/mt_c1601b0001_q.mtrl".to_string(), "files/bibomtrl/c1601b0001_v3.mtrl".to_string()),
				("chara/human/c1601/obj/body/b0001/material/v0004/mt_c1601b0001_q.mtrl".to_string(), "files/bibomtrl/c1601b0001_v4.mtrl".to_string()),
				("chara/human/c1601/obj/body/b0001/material/v0005/mt_c1601b0001_q.mtrl".to_string(), "files/bibomtrl/c1601b0001_v5.mtrl".to_string()),
				("chara/human/c1801/obj/body/b0001/material/v0001/mt_c1801b0001_q.mtrl".to_string(), "files/bibomtrl/c1801b0001.mtrl".to_string()),
			];
			
			_ = std::fs::create_dir(format!("{mod_path}/files/bibomtrl"));
			for (_, path) in &files {
				std::fs::copy(&path, format!("{mod_path}/{path}")).unwrap();
			}
		} else {
			files = Vec::new();
		}
		
		println!("{body_name}");
		println!("  Converting Gear");
		for race_id in body_type.race_ids() {
			println!("    Race {race_id:02}");
			files.append(&mut (1..9000).into_par_iter()
			// files.append(&mut (0204..=0204).into_par_iter()
				.map_init(|| physis::resource::SqPackResource::from_existing(&game_dir), |game, e| {
					// println!("{e}");
					let mut files = Vec::new();
					for typ in ["top", "dwn", "glv", "sho"] {
						let mut race_id = *race_id;
						let mut path = format!("chara/equipment/e{e:04}/model/c{race_id:02}01e{e:04}_{typ}.mdl");
						// println!("{path}");
						// let Some(mut bytes) = game.read(&path) else {continue};
						
						let Some(mut bytes) = ({
							let mut bytes = game.read(&path);
							
							// fhyur, mroe, or mlala inherit from mhyur if they dont have their own model
							if bytes.is_none() && (race_id == 02 || race_id == 09 || race_id == 11) {
								race_id = 01;
								path = format!("chara/equipment/e{e:04}/model/c{race_id:02}01e{e:04}_{typ}.mdl");
								bytes = game.read(&path);
								
								// swap to this once i figure out how to scale perfectly for EQDP
								// bytes = game.read(&format!("chara/equipment/e{e:04}/model/c0101e{e:04}_{typ}.mdl"));
							}
							
							bytes
						}) else {continue};
						
						if !fix_model(&mut bytes, body_type) {continue};
						
						let dir_path = format!("{mod_path}/files/{e:04}");
						_ = std::fs::create_dir(&dir_path);
						let file_path = format!("{dir_path}/{race_id:02}_{typ}.mdl");
						let mut file = File::create(&file_path).unwrap();
						file.write_all(&bytes).unwrap();
						files.push((path, format!("files/{e:04}/{race_id:02}_{typ}.mdl")));
					}
					
					files
				})
				.flatten()
				.collect::<Vec<_>>());
		}
		
		println!("  Writing meta");
		
		let files_str = files
			.into_iter()
			.map(|(a, b)| format!("\"{a}\":\"{b}\""))
			.collect::<Vec<_>>()
			.join(",\n\t\t");
		std::fs::write(format!("{mod_path}/default_mod.json"), format!(r#"{{
	"Version": 0,
	"FileSwaps": {{}},
	"Manipulations": [],
	"Files": {{
		{files_str}
	}}
}}"#)).unwrap();
		
		std::fs::write(format!("{mod_path}/meta.json"), format!(r#"{{
	"FileVersion": 3,
	"Name": "{mod_name}",
	"Author": "Sevii",
	"Description": "{}",
	"Image": "",
	"Version": "{version}",
	"Website": "https://github.com/Sevii77/ffxiv_uv_asymetrizer",
	"ModTags": ["{body_name}", "Asymmetrical", "Vanilla", "Gear"],
	"DefaultPreferredItems": []
}}"#, DESC.replace("\n", "\\n"))).unwrap();
		
		// return;
	}
}

fn fix_model(bytes: &mut Vec<u8>, body_type: BodyType) -> bool {
	let Ok(mdl) = mdl::Mdl::read_le(&mut std::io::Cursor::new(&bytes)) else {return false};
	let string_buf_offset = size_of::<mdl::HeaderRaw>()
		+ size_of::<[mdl::VertexElementRaw; 17]>() * mdl.header.vertex_declaration_count as usize
		+ 8;
	
	let mut skin_material_index = None;
	for (i, mat_offset) in mdl.material_string_offset.into_iter().enumerate() {
		let mut mat = mdl.strings_buf[mat_offset as usize..].null_terminated().unwrap();
		if !is_body_material(&mat) {continue};
		
		// let Some(body_type) = BodyType::from_partial_material_path(&mat) else {
		// 	println!("Failed getting body type from {mat}");
		// 	continue;
		// };
		
		let mat_len = mat.len();
		body_type.fix_material(&mut mat);
		let mut mat_buf = mat.as_bytes().to_owned();
		mat_buf.push(0);
		bytes.splice(string_buf_offset + mat_offset as usize..string_buf_offset + mat_offset as usize + mat_len + 1, mat_buf);
		
		skin_material_index = Some(i as u16);
	}
	
	let Some(skin_material_index) = skin_material_index else {return false};
	
	let mut rw = std::io::Cursor::new(bytes);
	macro_rules! r {
		(move $c:expr) => {{
			rw.seek_relative($c as i64).unwrap()
		}};
		
		(seek $c:expr) => {{
			rw.seek(::std::io::SeekFrom::Start($c as u64)).unwrap()
		}};
		
		(Vec<$e:ty>, $c:expr) => {{
			let mut v = Vec::with_capacity($c as usize);
			for _ in 0..$c {
				v.push(<$e>::read_options(&mut rw, binrw::Endian::Little, ()).unwrap());
			}
			v
		}};
		
		(f16) => {{
			half::f16::from_bits(r!(u16)).to_f32()
		}};
		
		($e:ty) => {{
			<$e>::read_options(&mut rw, binrw::Endian::Little, ()).unwrap()
		}};
	}
	
	for (lod_index, lod_raw) in mdl.lods.iter().enumerate() {
		for mesh_index in lod_raw.mesh_index as usize..(lod_raw.mesh_index + lod_raw.mesh_count) as usize {
			let mesh_raw = &mdl.meshes[mesh_index];
			if mesh_raw.material_index != skin_material_index {continue}
			
			let vertex_decl = &mdl.vertex_declerations[mesh_index];
			
			// read vertex positions first, so we can know if the entire triangle is left or right
			let mut positions = Vec::new();
			for stream in 0..3u8 {
				r!(seek mdl.header.vertex_offsets[lod_index] as u64 + mesh_raw.vertex_buffer_offset[stream as usize] as u64);
				for _vertex_index in 0..mesh_raw.vertex_count as usize {
					for decl in vertex_decl {
						if decl.stream == 255 {break}
						if decl.stream != stream {continue}
						
						let val = match decl.typ {
							mdl::VertexTypeRaw::F32x1 => [r!(f32), 0.0, 0.0, 0.0],
							mdl::VertexTypeRaw::F32x2 => [r!(f32), r!(f32), 0.0, 0.0],
							mdl::VertexTypeRaw::F32x3 => [r!(f32), r!(f32), r!(f32), 0.0],
							mdl::VertexTypeRaw::F32x4 => [r!(f32), r!(f32), r!(f32), r!(f32)],
							mdl::VertexTypeRaw::U8x4  => [r!(u8) as f32, r!(u8) as f32, r!(u8) as f32, r!(u8) as f32],
							mdl::VertexTypeRaw::F8x4  => [r!(u8) as f32 / 255.0, r!(u8) as f32 / 255.0, r!(u8) as f32 / 255.0, r!(u8) as f32 / 255.0],
							mdl::VertexTypeRaw::F16x2 => [r!(f16), r!(f16), 0.0, 0.0],
							mdl::VertexTypeRaw::F16x4 => [r!(f16), r!(f16), r!(f16), r!(f16)],
							mdl::VertexTypeRaw::U16x2 => [r!(u16) as f32, r!(u16) as f32, 0.0, 0.0],
							mdl::VertexTypeRaw::U16x4 => [r!(u16) as f32, r!(u16) as f32, r!(u16) as f32, r!(u16) as f32],
						};
						
						if matches!(decl.usage, mdl::VertexUsageRaw::Position) {
							positions.push(val);
						}
					}
				}
			}
			
			// calculate adv triangle center for each position, not great for shared vertices but the center ones
			// (the ones we care about), are duplicated already. so its fine
			let mut centers = vec![[0.0; 3]; positions.len()];
			for submesh_index in mesh_raw.submesh_index as usize..(mesh_raw.submesh_index + mesh_raw.submesh_count) as usize {
				let submesh_raw = &mdl.submeshes[submesh_index];
				r!(seek mdl.header.index_offsets[lod_index] as u64 + submesh_raw.index_offset as u64 * 2);
				let indices = r!(Vec<u16>, submesh_raw.index_count);
				for triangle in indices.chunks_exact(3) {
					let avg = [
						(positions[triangle[0] as usize][0] + positions[triangle[1] as usize][0] + positions[triangle[2] as usize][0]) / 3.0,
						(positions[triangle[0] as usize][1] + positions[triangle[1] as usize][1] + positions[triangle[2] as usize][1]) / 3.0,
						(positions[triangle[0] as usize][2] + positions[triangle[1] as usize][2] + positions[triangle[2] as usize][2]) / 3.0,
					];
					
					for index in triangle {
						centers[*index as usize] = avg.clone();
					}
				}
			}
			
			// modify uvs
			for stream in 0..3u8 {
				r!(seek mdl.header.vertex_offsets[lod_index] as u64 + mesh_raw.vertex_buffer_offset[stream as usize] as u64);
				for vertex_index in 0..mesh_raw.vertex_count as usize {
					for decl in vertex_decl {
						if decl.stream == 255 {break}
						if decl.stream != stream {continue}
						
						let mut val = match decl.typ {
							mdl::VertexTypeRaw::F32x1 => [r!(f32), 0.0, 0.0, 0.0],
							mdl::VertexTypeRaw::F32x2 => [r!(f32), r!(f32), 0.0, 0.0],
							mdl::VertexTypeRaw::F32x3 => [r!(f32), r!(f32), r!(f32), 0.0],
							mdl::VertexTypeRaw::F32x4 => [r!(f32), r!(f32), r!(f32), r!(f32)],
							mdl::VertexTypeRaw::U8x4  => [r!(u8) as f32, r!(u8) as f32, r!(u8) as f32, r!(u8) as f32],
							mdl::VertexTypeRaw::F8x4  => [r!(u8) as f32 / 255.0, r!(u8) as f32 / 255.0, r!(u8) as f32 / 255.0, r!(u8) as f32 / 255.0],
							mdl::VertexTypeRaw::F16x2 => [r!(f16), r!(f16), 0.0, 0.0],
							mdl::VertexTypeRaw::F16x4 => [r!(f16), r!(f16), r!(f16), r!(f16)],
							mdl::VertexTypeRaw::U16x2 => [r!(u16) as f32, r!(u16) as f32, 0.0, 0.0],
							mdl::VertexTypeRaw::U16x4 => [r!(u16) as f32, r!(u16) as f32, r!(u16) as f32, r!(u16) as f32],
						};
						
						if matches!(decl.usage, mdl::VertexUsageRaw::Uv) {
							const EPSILON: f32 = 0.00001f32;
							let pos = centers.get(vertex_index).unwrap();
							let is_high = val[0] > 1.0;
							let mut u = if is_high {val[0] - 1.0} else {val[0]};
							
							if pos[0] > EPSILON {
								u = u / 2.0 + 0.5;
							} else if pos[0] < EPSILON {
								u = 0.5 - u / 2.0;
							} else {
								u = 0.5;
								println!("CENTER OF TRIANGLE IS DEAD CENTER!!1!")
							}
							
							val[0] = if is_high {u + 1.0} else {u};
							
							match decl.typ {
								mdl::VertexTypeRaw::F32x2 => {
									r!(move -8);
									let v = &val[0..2];
									let v = unsafe{std::slice::from_raw_parts(v.as_ptr() as *const u8, 8)};
									rw.write_all(v).unwrap();
								}
								
								mdl::VertexTypeRaw::F32x4 => {
									r!(move -16);
									let v = &val[0..4];
									let v = unsafe{std::slice::from_raw_parts(v.as_ptr() as *const u8, 16)};
									rw.write_all(v).unwrap();
								}
								
								mdl::VertexTypeRaw::F16x2 => {
									r!(move -4);
									let v = [half::f16::from_f32(val[0]), half::f16::from_f32(val[1])];
									let v = unsafe{std::slice::from_raw_parts(v.as_ptr() as *const u8, 4)};
									rw.write_all(v).unwrap();
								}
								
								mdl::VertexTypeRaw::F16x4 => {
									r!(move -8);
									let v = [half::f16::from_f32(val[0]), half::f16::from_f32(val[1]), half::f16::from_f32(val[2]), half::f16::from_f32(val[3])];
									let v = unsafe{std::slice::from_raw_parts(v.as_ptr() as *const u8, 8)};
									rw.write_all(v).unwrap();
								}
								
								t => panic!("UV WAS A WEIRD FORMAT!!! {t:?}")
							};
						}
					}
				}
			}
		}
	}
	
	return true
}

fn is_body_material(material_path: &str) -> bool {
	let type1 = &material_path[4..=4];
	let type2 = &material_path[9..=9];
	
	type1 == "c" && type2 == "b"
}

#[derive(Copy, Clone, PartialEq)]
enum BodyType {
	TallFemale,
	TallMale,
	BigMale,
	Lalafell,
}

impl BodyType {
	// pub fn from_id(id: u8) -> Option<Self> {
	// 	Some(match id {
	// 		01 => Self::TallMale,
	// 		02 => Self::TallFemale,
	// 		03 => Self::TallMale,
	// 		04 => Self::TallFemale,
	// 		05 => Self::TallMale,
	// 		06 => Self::TallFemale,
	// 		07 => Self::TallMale,
	// 		08 => Self::TallFemale,
	// 		09 => Self::BigMale,
	// 		10 => Self::TallFemale,
	// 		11 => Self::Lalafell,
	// 		12 => Self::Lalafell,
	// 		13 => Self::TallMale,
	// 		14 => Self::TallFemale,
	// 		15 => Self::BigMale,
	// 		16 => Self::TallFemale,
	// 		17 => Self::TallMale,
	// 		18 => Self::TallFemale,
	// 		_ => return None,
	// 	})
	// }
	
	// pub fn from_full_path(path: &str) -> Option<Self> {
	// 	let p = path.find("/c")?;
	// 	let id = &path[p + 2..p + 4];
	// 	let id = u8::from_str_radix(id, 10).ok()?;
	// 	Self::from_id(id)
	// }
	
	// pub fn from_partial_material_path(path: &str) -> Option<Self> {
	// 	let id = &path[5..7];
	// 	let id = u8::from_str_radix(id, 10).ok()?;
	// 	Self::from_id(id)
	// }
	
	pub fn fix_material(&self, mat: &mut String) {
		match self {
			// BodyType::TallFemale => *mat = mat.replace("_a.mtrl", "_bibo.mtrl"),
			// we redirect _q > _bibo, since we do a simple string replacement,
			// it might fuck up since we arent changing the string offsets of anything
			Self::TallFemale => *mat = mat.replace("_a.mtrl", "_q.mtrl"),
			Self::TallMale   => *mat = mat.replace("_a.mtrl", "_b.mtrl"),
			Self::BigMale    => *mat = mat.replace("_a.mtrl", "_b.mtrl"),
			Self::Lalafell   => *mat = mat.replace("_a.mtrl", "_g.mtrl"),
		}
	}
	
	pub fn race_ids(&self) -> &'static [u8] {
		match self {
			Self::TallFemale => &[02, 04, 06, 08, 10, 14, 16, 18],
			Self::TallMale   => &[01, 03, 05, 07, 13, 17],
			Self::BigMale    => &[09, 15],
			Self::Lalafell   => &[11, 12],
		}
	}
	
	pub fn name(&self) -> &'static str {
		match self {
			Self::TallFemale => "Bibo",
			Self::TallMale   => "The Body SE",
			Self::BigMale    => "HR3",
			Self::Lalafell   => "Otopop",
		}
	}
}