extern crate gl;
extern crate libc;
extern crate std;

use gl::types::*;
use std::io;
use std::io::{ Error, ErrorKind };
use std::io::prelude::*;
use std::fs::File;

pub struct Texture {
	id: GLuint,
}

impl Texture {
	pub fn new() -> Texture {
		Texture {
			id: 0,
		}
	}

	pub fn begin(&self) {
		unsafe {
			gl::BindTexture(gl::TEXTURE_2D, self.id);
		}
	}

	pub fn end(&self) {
		unsafe {
			gl::BindTexture(gl::TEXTURE_2D, 0);
		}
	}

	pub fn load(&mut self, file_path: &str) {
		if file_path.to_lowercase().ends_with(".bmp") {
			self.load_bmp(file_path);
		} else if file_path.to_lowercase().ends_with(".dds") {
			self.load_dds(file_path);
		} else {
			println!("Not a correct image format!");
		}
	}

	// Loading bmp manually for educational purposes use dds
	fn load_bmp(&mut self, file_path: &str) -> io::Result<()> {
		let mut file = try!(File::open(file_path));
		let mut header: [u8; 54] = [0; 54];

		// Header
		try!(file.read(&mut header));

		if (header[0] != 66) || (header[1] != 77) {
			return Err(Error::new(ErrorKind::Other, "Not a bmp file!"));
 		}

		let mut image_size: usize = 0;
		let mut width = 0;
		let mut height = 0;

		unsafe {
			let raw_width = [header[0x12], header[0x13], header[0x14], header[0x15]];
			width = std::mem::transmute::<[u8; 4], i32>(raw_width);
			let raw_height = [header[0x16], header[0x17], header[0x18], header[0x19]];
			height = std::mem::transmute::<[u8; 4], i32>(raw_height);
			let raw_image_size = [header[0x22], header[0x23], header[0x24], header[0x25]];
			image_size = std::mem::transmute::<[u8; 4], u32>(raw_image_size) as usize;
		}

		if image_size == 0 {
			image_size = (width * height * 3) as usize;
		}

		// Data
		let mut data = vec![0; image_size];
		try!(file.read(&mut data)); // Read from where header ended

		// Give data to OpenGL and create texture
		unsafe {
			gl::GenTextures(1, &mut self.id);
			gl::BindTexture(gl::TEXTURE_2D, self.id);

			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
    		gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);

			gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as i32, width, height,
				0, gl::BGR, gl::UNSIGNED_BYTE, std::mem::transmute(&data[0]));

			gl::GenerateMipmap(gl::TEXTURE_2D);

			data.clear();
			gl::BindTexture(gl::TEXTURE_2D, 0);
		}

		Ok(())
	}

	// Compress png images with AMDCompress(CLI) to dds
	// Only use compressed images (DXT1 = BC1), (DXT3 = BC2), (DXT5 = BC3)
	// For sprites compress to DXT5 for alpha (gradient) channel
	fn load_dds(&mut self, file_path: &str) -> io::Result<()> {
		let mut file = try!(File::open(file_path));
		let mut header: [u8; 128] = [0; 128];

		// Header
		try!(file.read(&mut header));
		if (header[0] != 0x44) || (header[1] != 0x44) ||
			(header[2] != 0x53) || (header[3] != 0x20) {
			return Err(Error::new(ErrorKind::Other, "Not a dds file!"));
 		}

		let raw_height = [header[12], header[13], header[14], header[15]];
		let mut height = unsafe { std::mem::transmute::<[u8; 4], i32>(raw_height) };

		let raw_width = [header[16], header[17], header[18], header[19]];
		let mut width = unsafe { std::mem::transmute::<[u8; 4], i32>(raw_width) };

		let raw_linear_size = [header[20], header[21], header[22], header[23]];
		let linear_size = unsafe { std::mem::transmute::<[u8; 4], u32>(raw_linear_size) };

		let raw_mipmap_count = [header[28], header[29], header[30], header[31]];
		let mipmap_count = unsafe { std::mem::transmute::<[u8; 4], u32>(raw_mipmap_count) };

		let raw_four_cc = [header[84], header[85], header[86], header[87]];
		let four_cc = unsafe { std::mem::transmute::<[u8; 4], u32>(raw_four_cc) };

		// Data
		let image_size = if mipmap_count > 1 { linear_size * 2 } else { linear_size } as usize;
		let mut data = vec![0; image_size];
		try!(file.read(&mut data)); // Read from where header ended

		const FOURCC_DXT1: u32 = 0x31545844;
		const FOURCC_DXT3: u32 = 0x33545844;
		const FOURCC_DXT5: u32 = 0x35545844;
		const COMPRESSED_RGBA_S3TC_DXT1_EXT: u32 = 0x83F1;
		const COMPRESSED_RGBA_S3TC_DXT3_EXT: u32= 0x83F2;
		const COMPRESSED_RGBA_S3TC_DXT5_EXT: u32= 0x83F3;

		let mut format: u32 = 0;
		let mut block_size = 16;

		match four_cc {
			FOURCC_DXT1 => { format = COMPRESSED_RGBA_S3TC_DXT1_EXT; block_size = 8; },
			FOURCC_DXT3 => { format = COMPRESSED_RGBA_S3TC_DXT3_EXT; },
			FOURCC_DXT5 => { format = COMPRESSED_RGBA_S3TC_DXT5_EXT; },
			_ => return Err(Error::new(ErrorKind::Other, "No DXTn specified.")),
		};

		// Give data to OpenGL and create texture
		unsafe {
			gl::GenTextures(1, &mut self.id);
			gl::BindTexture(gl::TEXTURE_2D, self.id);
			gl::PixelStorei(gl::UNPACK_ALIGNMENT,1);

			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);	// REPEAT
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);	// REPEAT
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
		}

		// Load mipmaps

		let mut level = 0;
		let mut offset = 0;

		while level < mipmap_count && width > 0 && height > 0 {
			let size = ((width+3)/4)*((height+3)/4)*block_size;
			unsafe {
				gl::CompressedTexImage2D(gl::TEXTURE_2D, level as i32, format, width, height,
					0, size, std::mem::transmute(&data[offset as usize]));
			}

			offset += size;
			width /= 2;
			height /= 2;

			level += 1;
		}

		data.clear();
		unsafe {
			gl::BindTexture(gl::TEXTURE_2D, 0);
		}

		Ok(())
	}
}