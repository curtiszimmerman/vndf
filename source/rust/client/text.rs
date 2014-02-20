use std::hashmap::HashMap;
use std::libc;
use std::ptr;
use std::str;

use freetype::freetype::{
	FT_Face,
	FT_Get_Char_Index,
	FT_GlyphSlot,
	FT_Init_FreeType,
	FT_Library,
	FT_LOAD_DEFAULT,
	FT_Load_Glyph,
	FT_New_Face,
	FT_Render_Glyph,
	FT_RENDER_MODE_NORMAL,
	FT_Set_Pixel_Sizes,
	struct_FT_GlyphSlotRec_};
use gl;

use texture::Texture;


pub fn load_font() -> HashMap<~str, Texture> {
	let chars = [
		'a',
		'b',
		'c',
		'd',
		'e',
		'f',
		'g',
		'h',
		'i',
		'j',
		'k',
		'l',
		'm',
		'n',
		'o',
		'p',
		'q',
		'r',
		's',
		't',
		'u',
		'v',
		'w',
		'x',
		'y',
		'z',
		'A',
		'B',
		'C',
		'D',
		'E',
		'F',
		'G',
		'H',
		'I',
		'J',
		'K',
		'L',
		'M',
		'N',
		'O',
		'P',
		'Q',
		'R',
		'S',
		'T',
		'U',
		'V',
		'W',
		'X',
		'Y',
		'Z' ];

	let mut font = HashMap::new();
	for &c in chars.iter() {
		font.insert(str::from_char(c), load_char_as_texture(c));
	}

	font
}

pub fn load_char_as_texture(c: char) -> Texture {
	unsafe {
		let freetype: FT_Library = ptr::null();
		let init_error = FT_Init_FreeType(&freetype);
		assert!(init_error == 0);

		let mut font_face: FT_Face = ptr::null();
		let face_error = FT_New_Face(
				freetype,
				"fonts/amble/Amble-Bold.ttf".as_ptr() as *i8,
				0,
				&mut font_face);
		assert!(face_error == 0);

		let pixel_error = FT_Set_Pixel_Sizes(
			font_face,
			0,
			16);
		assert!(pixel_error == 0);

		let glyph_index = FT_Get_Char_Index(font_face, c as u64);

		let glyph_error = FT_Load_Glyph(
			font_face,
			glyph_index,
			FT_LOAD_DEFAULT as i32);
		assert!(glyph_error == 0);

		let render_error = FT_Render_Glyph(
			(*font_face).glyph as FT_GlyphSlot,
			FT_RENDER_MODE_NORMAL);
		assert!(render_error == 0);

		// Generate texture names.
		let mut texture_name: gl::types::GLuint = 0;
		gl::GenTextures(1, &mut texture_name);

		gl::BindTexture(
			gl::TEXTURE_2D,
			texture_name);

		// Configure texture.
		gl::TexParameteri(
			gl::TEXTURE_2D,
			gl::TEXTURE_MIN_FILTER,
			gl::NEAREST as i32);

		let bitmap =
			(*(((*font_face).glyph) as *struct_FT_GlyphSlotRec_)).bitmap;

		// Bind image data to texture name.
		gl::TexImage2D(
			gl::TEXTURE_2D,
			0,
			gl::ALPHA8 as i32,
			bitmap.width,
			bitmap.rows,
			0,
			gl::ALPHA,
			gl::UNSIGNED_BYTE,
			bitmap.buffer as *libc::c_void);

		Texture {
			name  : texture_name,
			width : bitmap.width as uint,
			height: bitmap.rows as uint}
	}
}
