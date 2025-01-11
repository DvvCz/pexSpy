use nanoserde::{DeRon, SerRon};
use std::io::{Read, Write};

struct Reader<'a> {
	cursor: std::io::Cursor<&'a [u8]>,
}

macro_rules! arg {
	// Can also cast a number into a bool.
	($self:expr, Bool) => {
		match $self.read_variable_data()? {
			VariableData::Bool(b) => b,
			VariableData::Int(i) => i != 0,
			got => panic!("Expected Bool, got {:?}", got),
		}
	};

	($self:expr, Value) => {
		$self.read_variable_data()?
	};

	($self:expr, $k:ident) => {
		match $self.read_variable_data()? {
			VariableData::$k(inner) => inner,
			got => panic!("Expected {}, got {:?}", stringify!($k), got),
		}
	};
}

impl<'a> Reader<'a> {
	pub fn new(b: &'a [u8]) -> Self {
		Self {
			cursor: std::io::Cursor::new(b),
		}
	}

	pub fn read_u8(&mut self) -> std::io::Result<u8> {
		let mut buf = [0; 1];
		self.cursor.read_exact(&mut buf)?;
		Ok(buf[0])
	}

	pub fn read_u16(&mut self) -> std::io::Result<u16> {
		let mut buf = [0; 2];
		self.cursor.read_exact(&mut buf)?;
		Ok(u16::from_be_bytes(buf))
	}

	pub fn read_u32(&mut self) -> std::io::Result<u32> {
		let mut buf = [0; 4];
		self.cursor.read_exact(&mut buf)?;
		Ok(u32::from_be_bytes(buf))
	}

	pub fn read_u64(&mut self) -> std::io::Result<u64> {
		let mut buf = [0; 8];
		self.cursor.read_exact(&mut buf)?;
		Ok(u64::from_be_bytes(buf))
	}

	pub fn read_i32(&mut self) -> std::io::Result<i32> {
		let mut buf = [0; 4];
		self.cursor.read_exact(&mut buf)?;
		Ok(i32::from_be_bytes(buf))
	}

	pub fn read_f32(&mut self) -> std::io::Result<f32> {
		let mut buf = [0; 4];
		self.cursor.read_exact(&mut buf)?;
		Ok(f32::from_be_bytes(buf))
	}

	pub fn read_wstring(&mut self) -> std::io::Result<String> {
		let len = self.read_u16()? as usize;

		let mut buf = vec![0; len];
		self.cursor.read_exact(&mut buf)?;

		Ok(String::from_utf8_lossy(&buf).into_owned())
	}

	pub fn read_instruction(&mut self) -> PexResult<Instruction> {
		match self.read_u8()? {
			0 => Ok(Instruction::NOP),
			1 => Ok(Instruction::IADD(
				arg!(self, Ident),
				arg!(self, Value),
				arg!(self, Value),
			)),
			2 => Ok(Instruction::FADD(
				arg!(self, Ident),
				arg!(self, Value),
				arg!(self, Value),
			)),
			3 => Ok(Instruction::ISUB(
				arg!(self, Ident),
				arg!(self, Value),
				arg!(self, Value),
			)),
			4 => Ok(Instruction::FSUB(
				arg!(self, Ident),
				arg!(self, Value),
				arg!(self, Value),
			)),
			5 => Ok(Instruction::IMUL(
				arg!(self, Ident),
				arg!(self, Value),
				arg!(self, Value),
			)),
			6 => Ok(Instruction::FMUL(
				arg!(self, Ident),
				arg!(self, Value),
				arg!(self, Value),
			)),
			7 => Ok(Instruction::IDIV(
				arg!(self, Ident),
				arg!(self, Value),
				arg!(self, Value),
			)),
			8 => Ok(Instruction::FDIV(
				arg!(self, Ident),
				arg!(self, Value),
				arg!(self, Value),
			)),
			9 => Ok(Instruction::IMOD(
				arg!(self, Ident),
				arg!(self, Value),
				arg!(self, Value),
			)),
			10 => Ok(Instruction::NOT(arg!(self, Ident), arg!(self, Value))),
			11 => Ok(Instruction::INEG(arg!(self, Ident), arg!(self, Value))),
			12 => Ok(Instruction::FNEG(arg!(self, Ident), arg!(self, Value))),
			13 => Ok(Instruction::ASSIGN(arg!(self, Ident), arg!(self, Value))),
			14 => Ok(Instruction::CAST(arg!(self, Ident), arg!(self, Value))),
			15 => Ok(Instruction::CMP_EQ(
				arg!(self, Ident),
				arg!(self, Value),
				arg!(self, Value),
			)),
			16 => Ok(Instruction::CMP_LT(
				arg!(self, Ident),
				arg!(self, Value),
				arg!(self, Value),
			)),
			17 => Ok(Instruction::CMP_LE(
				arg!(self, Ident),
				arg!(self, Value),
				arg!(self, Value),
			)),
			18 => Ok(Instruction::CMP_GT(
				arg!(self, Ident),
				arg!(self, Value),
				arg!(self, Value),
			)),
			19 => Ok(Instruction::CMP_GE(
				arg!(self, Ident),
				arg!(self, Value),
				arg!(self, Value),
			)),
			20 => Ok(Instruction::JMP(arg!(self, Value))),
			21 => Ok(Instruction::JMPT(arg!(self, Value), arg!(self, Value))),
			22 => Ok(Instruction::JMPF(arg!(self, Value), arg!(self, Value))),
			23 => Ok(Instruction::CALLMETHOD(
				arg!(self, Ident),
				arg!(self, Value),
				arg!(self, Ident),
				{
					let count = arg!(self, Int);
					(0..count)
						.map(|_| self.read_variable_data())
						.collect::<PexResult<Vec<_>>>()?
				},
			)),
			24 => Ok(Instruction::CALLPARENT(
				arg!(self, Ident), // String?
				arg!(self, Ident),
				{
					let count = arg!(self, Int);
					(0..count)
						.map(|_| self.read_variable_data())
						.collect::<PexResult<Vec<_>>>()?
				},
			)),
			25 => Ok(Instruction::CALLSTATIC(
				arg!(self, Ident), // String?
				arg!(self, Ident), // String?
				arg!(self, Ident),
				{
					let count = arg!(self, Int);
					(0..count)
						.map(|_| self.read_variable_data())
						.collect::<PexResult<Vec<_>>>()?
				},
			)),
			26 => Ok(Instruction::RETURN(self.read_variable_data()?)),
			27 => Ok(Instruction::STRCAT(
				arg!(self, Ident),
				arg!(self, Value),
				arg!(self, Value),
			)),
			28 => Ok(Instruction::PROPGET(
				arg!(self, Ident), // String ?
				arg!(self, Ident),
				arg!(self, Ident),
			)),
			29 => Ok(Instruction::PROPSET(
				arg!(self, Ident),
				arg!(self, Ident),
				arg!(self, Value),
			)),
			30 => Ok(Instruction::ARRAY_CREATE(
				arg!(self, Ident),
				arg!(self, Int) as u32,
			)),
			31 => Ok(Instruction::ARRAY_LENGTH(
				arg!(self, Ident),
				arg!(self, Ident),
			)),
			32 => Ok(Instruction::ARRAY_GETELEMENT(
				arg!(self, Ident),
				arg!(self, Ident),
				arg!(self, Value),
			)),
			33 => Ok(Instruction::ARRAY_SETELEMENT(
				arg!(self, Ident),
				arg!(self, Value),
				arg!(self, Value),
			)),
			34 => Ok(Instruction::ARRAY_FINDELEMENT(
				arg!(self, Ident),
				arg!(self, Ident),
				arg!(self, Value),
				arg!(self, Int),
			)),
			35 => Ok(Instruction::ARRAY_RFINDELEMENT(
				arg!(self, Ident),
				arg!(self, Ident),
				arg!(self, Value),
				arg!(self, Int),
			)),
			instr => todo!("Instruction {instr}"),
		}
	}

	pub fn read_variable_type(&mut self) -> std::io::Result<VariableType> {
		Ok(VariableType {
			name_idx: self.read_u16()?,
			type_idx: self.read_u16()?,
		})
	}

	pub fn read_variable_data(&mut self) -> PexResult<VariableData> {
		match self.read_u8()? {
			0 => Ok(VariableData::Null),
			1 => Ok(VariableData::Ident(self.read_u16()?)),
			2 => Ok(VariableData::String(self.read_u16()?)),
			3 => Ok(VariableData::Int(self.read_i32()?)),
			4 => Ok(VariableData::Float(self.read_f32()?)),
			5 => Ok(VariableData::Bool(self.read_u8()? != 0)),
			other => Err(PexError::InvalidVariableDataType(other)),
		}
	}

	pub fn read_property(&mut self) -> PexResult<Property> {
		let name_idx = self.read_u16()?;
		let type_idx = self.read_u16()?;
		let doc_string_idx = self.read_u16()?;
		let user_flags = self.read_u32()?;
		let flags = self.read_u8()?;

		let auto_var_name = if (flags & 4) != 0 {
			Some(self.read_u16()?)
		} else {
			None
		};

		let read_handler = if (flags & 5) == 1 {
			Some(self.read_function()?)
		} else {
			None
		};

		let write_handler = if (flags & 6) == 2 {
			Some(self.read_function()?)
		} else {
			None
		};

		Ok(Property {
			name_idx,
			type_idx,
			doc_string_idx,
			user_flags,
			flags,
			auto_var_name,
			read_handler,
			write_handler,
		})
	}

	pub fn read_function(&mut self) -> PexResult<Function> {
		let return_type_idx = self.read_u16()?;
		let doc_string_idx = self.read_u16()?;
		let user_flags = self.read_u32()?;
		let flags = self.read_u8()?;

		let params = {
			let count = self.read_u16()?;
			(0..count)
				.map(|_| self.read_variable_type())
				.collect::<std::io::Result<Vec<_>>>()?
		};

		let locals = {
			let count = self.read_u16()?;
			(0..count)
				.map(|_| self.read_variable_type())
				.collect::<std::io::Result<Vec<_>>>()?
		};

		let instructions = {
			let count = self.read_u16()?;
			(0..count)
				.map(|_| self.read_instruction())
				.collect::<PexResult<Vec<_>>>()?
		};

		Ok(Function {
			return_type_idx,
			doc_string_idx,
			user_flags,
			flags,
			params,
			locals,
			instructions,
		})
	}

	pub fn read_named_function(&mut self) -> PexResult<(u16, Function)> {
		let name_idx = self.read_u16()?;
		let function = self.read_function()?;

		Ok((name_idx, function))
	}

	pub fn read_state(&mut self) -> PexResult<State> {
		let name_idx = self.read_u16()?;
		let functions = {
			let count = self.read_u16()?;
			(0..count)
				.map(|_| self.read_named_function())
				.collect::<PexResult<Vec<_>>>()?
		};

		Ok(State {
			name_idx,
			functions,
		})
	}

	pub fn read_object_data(&mut self) -> PexResult<ObjectData> {
		let parent_name_idx = self.read_u16()?;
		let doc_string_idx = self.read_u16()?;
		let user_flags = self.read_u32()?;
		let auto_state_name_idx = self.read_u16()?;

		let variables = {
			let count = self.read_u16()?;
			(0..count)
				.map(|_| {
					let name_idx = self.read_u16()?;
					let type_idx = self.read_u16()?;
					let user_flags = self.read_u32()?;
					let data = self.read_variable_data()?;

					Ok((name_idx, type_idx, user_flags, data))
				})
				.collect::<PexResult<Vec<_>>>()?
		};

		let properties = {
			let count = self.read_u16()?;
			(0..count)
				.map(|_| self.read_property())
				.collect::<PexResult<Vec<_>>>()?
		};

		let states = {
			let count = self.read_u16()?;
			(0..count)
				.map(|_| self.read_state())
				.collect::<PexResult<Vec<_>>>()?
		};

		Ok(ObjectData {
			parent_name_idx,
			doc_string_idx,
			user_flags,
			auto_state_name_idx,
			variables,
			properties,
			states,
		})
	}

	pub fn read_debuginfo(&mut self) -> PexResult<Option<DebugInfo>> {
		let hasinfo = self.read_u8()? != 0;
		if !hasinfo {
			return Ok(None);
		}

		let modtime = self.read_u64()?;

		let functions = {
			let count = self.read_u16()?;
			(0..count)
				.map(|_| {
					let obj_name_idx = self.read_u16()?;
					let state_name_idx = self.read_u16()?;
					let fn_name_idx = self.read_u16()?;

					let fn_type = self.read_u8()?;

					let instructions = {
						let count = self.read_u16()?;
						(0..count)
							.map(|_| self.read_u16())
							.collect::<std::io::Result<Vec<_>>>()
					}?;

					Ok(DebugFunction {
						obj_name_idx,
						state_name_idx,
						fn_name_idx,
						fn_type,
						instructions,
					})
				})
				.collect::<std::io::Result<Vec<_>>>()?
		};

		Ok(Some(DebugInfo { modtime, functions }))
	}
}

#[derive(Debug, thiserror::Error)]
pub enum PexError {
	#[error("Invalid magic number")]
	Magic,

	#[error("Unsupported major version: {0}")]
	UnsupportedMajor(u8),

	#[error("Unsupported minor version: {0}")]
	UnsupportedMinor(u8),

	#[error("Unsupported game ID: {0}")]
	UnsupportedGame(u16),

	#[error("Invalid variable data type: {0}")]
	InvalidVariableDataType(u8),

	#[error("Instruction was passed incorrect argument type")]
	InvalidInstruction,

	#[error("IO error: {0}")]
	IO(#[from] std::io::Error),
}

pub type PexResult<T> = Result<T, PexError>;

#[derive(Debug, DeRon, SerRon)]
pub struct VariableType {
	name_idx: u16,
	type_idx: u16,
}

#[derive(Debug, DeRon, SerRon)]
pub enum VariableData {
	Null,
	Ident(u16),
	String(u16),
	Int(i32),
	Float(f32),
	Bool(bool),
}

#[derive(Debug, DeRon, SerRon)]
#[non_exhaustive]
#[allow(non_camel_case_types)]
pub enum Instruction {
	NOP,
	IADD(u16, VariableData, VariableData),
	FADD(u16, VariableData, VariableData),
	ISUB(u16, VariableData, VariableData),
	FSUB(u16, VariableData, VariableData),
	IMUL(u16, VariableData, VariableData),
	FMUL(u16, VariableData, VariableData),
	IDIV(u16, VariableData, VariableData),
	FDIV(u16, VariableData, VariableData),
	IMOD(u16, VariableData, VariableData),
	NOT(u16, VariableData),
	INEG(u16, VariableData),
	FNEG(u16, VariableData),
	ASSIGN(u16, VariableData),
	CAST(u16, VariableData),
	CMP_EQ(u16, VariableData, VariableData),
	CMP_LT(u16, VariableData, VariableData),
	CMP_LE(u16, VariableData, VariableData),
	CMP_GT(u16, VariableData, VariableData),
	CMP_GE(u16, VariableData, VariableData),
	JMP(VariableData),
	JMPT(VariableData, VariableData),
	JMPF(VariableData, VariableData),
	CALLMETHOD(u16, VariableData, u16, Vec<VariableData>),
	CALLPARENT(u16, u16, Vec<VariableData>),
	CALLSTATIC(u16, u16, u16, Vec<VariableData>),
	RETURN(VariableData),
	STRCAT(u16, VariableData, VariableData),
	PROPGET(u16, u16, u16),
	PROPSET(u16, u16, VariableData),
	ARRAY_CREATE(u16, u32),
	ARRAY_LENGTH(u16, u16),
	ARRAY_GETELEMENT(u16, u16, VariableData),
	ARRAY_SETELEMENT(u16, VariableData, VariableData),
	ARRAY_FINDELEMENT(u16, u16, VariableData, i32),
	ARRAY_RFINDELEMENT(u16, u16, VariableData, i32),
}

#[derive(Debug, DeRon, SerRon)]
pub struct Function {
	pub return_type_idx: u16,
	pub doc_string_idx: u16,
	pub user_flags: u32,
	pub flags: u8,
	pub params: Vec<VariableType>,
	pub locals: Vec<VariableType>,
	pub instructions: Vec<Instruction>,
}

#[derive(Debug, DeRon, SerRon)]
pub struct Property {
	name_idx: u16,
	type_idx: u16,
	doc_string_idx: u16,
	user_flags: u32,
	flags: u8,
	auto_var_name: Option<u16>,
	read_handler: Option<Function>,
	write_handler: Option<Function>,
}

#[derive(Debug, DeRon, SerRon)]
pub struct State {
	pub name_idx: u16,
	pub functions: Vec<(u16, Function)>,
}

#[derive(Debug, DeRon, SerRon)]
pub struct ObjectData {
	pub parent_name_idx: u16,
	pub doc_string_idx: u16,
	pub user_flags: u32,
	pub auto_state_name_idx: u16,
	pub variables: Vec<(u16, u16, u32, VariableData)>,
	pub properties: Vec<Property>,
	pub states: Vec<State>,
}

#[derive(Debug, DeRon, SerRon)]
pub struct DebugInfo {
	modtime: u64,
	functions: Vec<DebugFunction>,
}

#[derive(Debug, DeRon, SerRon)]
pub struct DebugFunction {
	obj_name_idx: u16,
	state_name_idx: u16,
	fn_name_idx: u16,
	fn_type: u8,
	instructions: Vec<u16>,
}

#[derive(Debug, DeRon, SerRon)]
pub struct Pex {
	pub major: u8,
	pub minor: u8,
	pub gameid: u16,
	pub comptime: u64,
	pub src: String,
	pub username: String,
	pub machine: String,
	pub stringtable: Vec<String>,
	pub debuginfo: Option<DebugInfo>,
	pub userflags: Vec<(u16, u8)>,
	pub objects: Vec<(u16, ObjectData)>,
}

pub fn parse(pex: &[u8]) -> PexResult<Pex> {
	let mut reader = Reader::new(pex);

	let magic = reader.read_u32()?;
	if magic != 0xFA57C0DE {
		return Err(PexError::Magic);
	}

	let major = reader.read_u8()?;
	if major != 3 {
		return Err(PexError::UnsupportedMajor(major));
	}

	let minor = reader.read_u8()?;
	if minor != 1 && minor != 2 {
		return Err(PexError::UnsupportedMinor(minor));
	}

	let gameid = reader.read_u16()?;
	if gameid != 1 {
		return Err(PexError::UnsupportedGame(gameid));
	}

	let comptime = reader.read_u64()?;
	let src = reader.read_wstring()?;
	let username = reader.read_wstring()?;
	let machine = reader.read_wstring()?;

	let stringtable = {
		let count = reader.read_u16()?;

		(0..count)
			.map(|_| reader.read_wstring())
			.collect::<std::io::Result<Vec<_>>>()?
	};

	let debuginfo = reader.read_debuginfo()?;

	let userflags = {
		let count = reader.read_u16()?;
		(0..count)
			.map(|_| {
				let name_idx = reader.read_u16()?;
				let flag_idx = reader.read_u8()?;
				Ok((name_idx, flag_idx))
			})
			.collect::<std::io::Result<Vec<_>>>()?
	};

	let objects = {
		let count = reader.read_u16()?;
		(0..count)
			.map(|_| {
				let name_idx = reader.read_u16()?;
				let _size = reader.read_u32()?;
				let data = reader.read_object_data()?;

				Ok((name_idx, data))
			})
			.collect::<PexResult<Vec<_>>>()?
	};

	Ok(Pex {
		major,
		minor,
		gameid,
		comptime,
		src,
		username,
		machine,
		stringtable,
		debuginfo,
		userflags,
		objects,
	})
}

pub struct Writer {
	cursor: std::io::Cursor<Vec<u8>>,
}

impl Writer {
	pub fn new() -> Self {
		Self {
			cursor: std::io::Cursor::new(Vec::new()),
		}
	}

	pub fn write_u8(&mut self, value: u8) -> std::io::Result<()> {
		self.cursor.write_all(&[value])
	}

	pub fn write_u16(&mut self, value: u16) -> std::io::Result<()> {
		self.cursor.write_all(&value.to_be_bytes())
	}

	pub fn write_u32(&mut self, value: u32) -> std::io::Result<()> {
		self.cursor.write_all(&value.to_be_bytes())
	}

	pub fn write_u64(&mut self, value: u64) -> std::io::Result<()> {
		self.cursor.write_all(&value.to_be_bytes())
	}

	pub fn write_i32(&mut self, value: i32) -> std::io::Result<()> {
		self.cursor.write_all(&value.to_be_bytes())
	}

	pub fn write_f32(&mut self, value: f32) -> std::io::Result<()> {
		self.cursor.write_all(&value.to_be_bytes())
	}

	pub fn write_wstring(&mut self, value: &str) -> std::io::Result<()> {
		self.write_u16(value.len() as u16)?;
		self.cursor.write_all(value.as_bytes())
	}

	pub fn write_instruction(&mut self, value: &Instruction) -> PexResult<()> {
		match value {
			Instruction::NOP => self.write_u8(0)?,
			Instruction::IADD(a, b, c) => {
				self.write_u8(1)?;
				self.write_variable_data(&VariableData::Ident(*a))?;
				self.write_variable_data(b)?;
				self.write_variable_data(c)?;
			}
			Instruction::FADD(a, b, c) => {
				self.write_u8(2)?;
				self.write_variable_data(&VariableData::Ident(*a))?;
				self.write_variable_data(b)?;
				self.write_variable_data(c)?;
			}
			Instruction::ISUB(a, b, c) => {
				self.write_u8(3)?;
				self.write_variable_data(&VariableData::Ident(*a))?;
				self.write_variable_data(b)?;
				self.write_variable_data(c)?;
			}
			Instruction::FSUB(a, b, c) => {
				self.write_u8(4)?;
				self.write_variable_data(&VariableData::Ident(*a))?;
				self.write_variable_data(b)?;
				self.write_variable_data(c)?;
			}
			Instruction::IMUL(a, b, c) => {
				self.write_u8(5)?;
				self.write_variable_data(&VariableData::Ident(*a))?;
				self.write_variable_data(b)?;
				self.write_variable_data(c)?;
			}
			Instruction::FMUL(a, b, c) => {
				self.write_u8(6)?;
				self.write_variable_data(&VariableData::Ident(*a))?;
				self.write_variable_data(b)?;
				self.write_variable_data(c)?;
			}
			Instruction::IDIV(a, b, c) => {
				self.write_u8(7)?;
				self.write_variable_data(&VariableData::Ident(*a))?;
				self.write_variable_data(b)?;
				self.write_variable_data(c)?;
			}
			Instruction::FDIV(a, b, c) => {
				self.write_u8(8)?;
				self.write_variable_data(&VariableData::Ident(*a))?;
				self.write_variable_data(b)?;
				self.write_variable_data(c)?;
			}
			Instruction::IMOD(a, b, c) => {
				self.write_u8(9)?;
				self.write_variable_data(&VariableData::Ident(*a))?;
				self.write_variable_data(b)?;
				self.write_variable_data(c)?;
			}
			Instruction::NOT(a, b) => {
				self.write_u8(10)?;
				self.write_variable_data(&VariableData::Ident(*a))?;
				self.write_variable_data(b)?;
			}
			Instruction::INEG(a, b) => {
				self.write_u8(11)?;
				self.write_variable_data(&VariableData::Ident(*a))?;
				self.write_variable_data(b)?;
			}
			Instruction::FNEG(a, b) => {
				self.write_u8(12)?;
				self.write_variable_data(&VariableData::Ident(*a))?;
				self.write_variable_data(b)?;
			}
			Instruction::ASSIGN(a, b) => {
				self.write_u8(13)?;
				self.write_variable_data(&VariableData::Ident(*a))?;
				self.write_variable_data(b)?;
			}
			Instruction::CAST(a, b) => {
				self.write_u8(14)?;
				self.write_variable_data(&VariableData::Ident(*a))?;
				self.write_variable_data(b)?;
			}
			Instruction::CMP_EQ(a, b, c) => {
				self.write_u8(15)?;
				self.write_variable_data(&VariableData::Ident(*a))?;
				self.write_variable_data(b)?;
				self.write_variable_data(c)?;
			}
			Instruction::CMP_LT(a, b, c) => {
				self.write_u8(16)?;
				self.write_variable_data(&VariableData::Ident(*a))?;
				self.write_variable_data(b)?;
				self.write_variable_data(c)?;
			}
			Instruction::CMP_LE(a, b, c) => {
				self.write_u8(17)?;
				self.write_variable_data(&VariableData::Ident(*a))?;
				self.write_variable_data(b)?;
				self.write_variable_data(c)?;
			}
			Instruction::CMP_GT(a, b, c) => {
				self.write_u8(18)?;
				self.write_variable_data(&VariableData::Ident(*a))?;
				self.write_variable_data(b)?;
				self.write_variable_data(c)?;
			}
			Instruction::CMP_GE(a, b, c) => {
				self.write_u8(19)?;
				self.write_variable_data(&VariableData::Ident(*a))?;
				self.write_variable_data(b)?;
				self.write_variable_data(c)?;
			}
			Instruction::JMP(a) => {
				self.write_u8(20)?;
				self.write_variable_data(a)?;
			}
			Instruction::JMPT(a, b) => {
				self.write_u8(21)?;
				self.write_variable_data(a)?;
				self.write_variable_data(b)?;
			}
			Instruction::JMPF(a, b) => {
				self.write_u8(22)?;
				self.write_variable_data(a)?;
				self.write_variable_data(b)?;
			}
			Instruction::CALLMETHOD(a, b, c, d) => {
				self.write_u8(23)?;
				self.write_variable_data(&VariableData::Ident(*a))?;
				self.write_variable_data(b)?;
				self.write_variable_data(&VariableData::Ident(*c))?;
				self.write_variable_data(&VariableData::Int(d.len() as i32))?;
				for arg in d {
					self.write_variable_data(arg)?;
				}
			}
			Instruction::CALLPARENT(a, b, c) => {
				self.write_u8(24)?;
				self.write_variable_data(&VariableData::Ident(*a))?;
				self.write_variable_data(&VariableData::Ident(*b))?;
				self.write_variable_data(&VariableData::Int(c.len() as i32))?;
				for arg in c {
					self.write_variable_data(arg)?;
				}
			}
			Instruction::CALLSTATIC(a, b, c, d) => {
				self.write_u8(25)?;
				self.write_variable_data(&VariableData::Ident(*a))?;
				self.write_variable_data(&VariableData::Ident(*b))?;
				self.write_variable_data(&VariableData::Ident(*c))?;
				self.write_variable_data(&VariableData::Int(d.len() as i32))?;
				for arg in d {
					self.write_variable_data(arg)?;
				}
			}
			Instruction::RETURN(a) => {
				self.write_u8(26)?;
				self.write_variable_data(a)?;
			}
			Instruction::STRCAT(a, b, c) => {
				self.write_u8(27)?;
				self.write_variable_data(&VariableData::Ident(*a))?;
				self.write_variable_data(b)?;
				self.write_variable_data(c)?;
			}
			Instruction::PROPGET(a, b, c) => {
				self.write_u8(28)?;
				self.write_variable_data(&VariableData::Ident(*a))?;
				self.write_variable_data(&VariableData::Ident(*b))?;
				self.write_variable_data(&VariableData::Ident(*c))?;
			}
			Instruction::PROPSET(a, b, c) => {
				self.write_u8(29)?;
				self.write_variable_data(&VariableData::Ident(*a))?;
				self.write_variable_data(&VariableData::Ident(*b))?;
				self.write_variable_data(c)?;
			}
			Instruction::ARRAY_CREATE(a, b) => {
				self.write_u8(30)?;
				self.write_variable_data(&VariableData::Ident(*a))?;
				self.write_variable_data(&VariableData::Int(*b as i32))?;
			}
			Instruction::ARRAY_LENGTH(a, b) => {
				self.write_u8(31)?;
				self.write_variable_data(&VariableData::Ident(*a))?;
				self.write_variable_data(&VariableData::Ident(*b))?;
			}
			Instruction::ARRAY_GETELEMENT(a, b, c) => {
				self.write_u8(32)?;
				self.write_variable_data(&VariableData::Ident(*a))?;
				self.write_variable_data(&VariableData::Ident(*b))?;
				self.write_variable_data(c)?;
			}
			Instruction::ARRAY_SETELEMENT(a, b, c) => {
				self.write_u8(33)?;
				self.write_variable_data(&VariableData::Ident(*a))?;
				self.write_variable_data(b)?;
				self.write_variable_data(c)?;
			}
			Instruction::ARRAY_FINDELEMENT(a, b, c, d) => {
				self.write_u8(34)?;
				self.write_variable_data(&VariableData::Ident(*a))?;
				self.write_variable_data(&VariableData::Ident(*b))?;
				self.write_variable_data(c)?;
				self.write_variable_data(&VariableData::Int(*d))?;
			}
			Instruction::ARRAY_RFINDELEMENT(a, b, c, d) => {
				self.write_u8(35)?;
				self.write_variable_data(&VariableData::Ident(*a))?;
				self.write_variable_data(&VariableData::Ident(*b))?;
				self.write_variable_data(c)?;
				self.write_variable_data(&VariableData::Int(*d))?;
			}
			_ => todo!("Write instruction {:?}", value),
		}
		Ok(())
	}

	pub fn write_variable_type(&mut self, value: &VariableType) -> std::io::Result<()> {
		self.write_u16(value.name_idx)?;
		self.write_u16(value.type_idx)
	}

	pub fn write_variable_data(&mut self, value: &VariableData) -> PexResult<()> {
		match value {
			VariableData::Null => self.write_u8(0)?,
			VariableData::Ident(i) => {
				self.write_u8(1)?;
				self.write_u16(*i)?;
			}
			VariableData::String(s) => {
				self.write_u8(2)?;
				self.write_u16(*s)?;
			}
			VariableData::Int(i) => {
				self.write_u8(3)?;
				self.write_i32(*i)?;
			}
			VariableData::Float(f) => {
				self.write_u8(4)?;
				self.write_f32(*f)?;
			}
			VariableData::Bool(b) => {
				self.write_u8(5)?;
				self.write_u8(if *b { 1 } else { 0 })?;
			}
		}
		Ok(())
	}

	pub fn write_property(&mut self, value: &Property) -> PexResult<()> {
		self.write_u16(value.name_idx)?;
		self.write_u16(value.type_idx)?;
		self.write_u16(value.doc_string_idx)?;
		self.write_u32(value.user_flags)?;
		self.write_u8(value.flags)?;

		if let Some(name) = value.auto_var_name {
			self.write_u16(name)?;
		}

		if let Some(handler) = &value.read_handler {
			self.write_function(handler)?;
		}

		if let Some(handler) = &value.write_handler {
			self.write_function(handler)?;
		}

		Ok(())
	}

	pub fn write_function(&mut self, value: &Function) -> PexResult<()> {
		self.write_u16(value.return_type_idx)?;
		self.write_u16(value.doc_string_idx)?;
		self.write_u32(value.user_flags)?;
		self.write_u8(value.flags)?;

		self.write_u16(value.params.len() as u16)?;
		for param in &value.params {
			self.write_variable_type(param)?;
		}

		self.write_u16(value.locals.len() as u16)?;
		for local in &value.locals {
			self.write_variable_type(local)?;
		}

		self.write_u16(value.instructions.len() as u16)?;
		for instruction in &value.instructions {
			self.write_instruction(instruction)?;
		}

		Ok(())
	}

	pub fn write_named_function(&mut self, value: &(u16, Function)) -> PexResult<()> {
		self.write_u16(value.0)?;
		self.write_function(&value.1)
	}

	pub fn write_state(&mut self, value: &State) -> PexResult<()> {
		self.write_u16(value.name_idx)?;

		self.write_u16(value.functions.len() as u16)?;
		for function in &value.functions {
			self.write_named_function(function)?;
		}

		Ok(())
	}

	pub fn write_object_data(&mut self, value: &ObjectData) -> PexResult<()> {
		self.write_u16(value.parent_name_idx)?;
		self.write_u16(value.doc_string_idx)?;
		self.write_u32(value.user_flags)?;
		self.write_u16(value.auto_state_name_idx)?;

		self.write_u16(value.variables.len() as u16)?;
		for var in &value.variables {
			self.write_u16(var.0)?;
			self.write_u16(var.1)?;
			self.write_u32(var.2)?;
			self.write_variable_data(&var.3)?;
		}

		self.write_u16(value.properties.len() as u16)?;
		for property in &value.properties {
			self.write_property(property)?;
		}

		self.write_u16(value.states.len() as u16)?;
		for state in &value.states {
			self.write_state(state)?;
		}

		Ok(())
	}

	pub fn write_debuginfo(&mut self, value: Option<&DebugInfo>) -> PexResult<()> {
		match value {
			None => self.write_u8(0)?,
			Some(debug) => {
				self.write_u8(1)?;
				self.write_u64(debug.modtime)?;

				self.write_u16(debug.functions.len() as u16)?;
				for func in &debug.functions {
					self.write_u16(func.obj_name_idx)?;
					self.write_u16(func.state_name_idx)?;
					self.write_u16(func.fn_name_idx)?;
					self.write_u8(func.fn_type)?;

					self.write_u16(func.instructions.len() as u16)?;
					for instruction in &func.instructions {
						self.write_u16(*instruction)?;
					}
				}
			}
		}
		Ok(())
	}
}

pub fn assemble(pex: &Pex) -> PexResult<Vec<u8>> {
	let mut writer = Writer::new();

	writer.write_u32(0xFA57C0DE)?;
	writer.write_u8(pex.major)?;
	writer.write_u8(pex.minor)?;
	writer.write_u16(pex.gameid)?;
	writer.write_u64(pex.comptime)?;
	writer.write_wstring(&pex.src)?;
	writer.write_wstring(&pex.username)?;
	writer.write_wstring(&pex.machine)?;

	writer.write_u16(pex.stringtable.len() as u16)?;
	for s in &pex.stringtable {
		writer.write_wstring(s)?;
	}

	writer.write_debuginfo(pex.debuginfo.as_ref())?;

	writer.write_u16(pex.userflags.len() as u16)?;
	for flag in &pex.userflags {
		writer.write_u16(flag.0)?;
		writer.write_u8(flag.1)?;
	}

	writer.write_u16(pex.objects.len() as u16)?;
	for obj in &pex.objects {
		writer.write_u16(obj.0)?;
		let start = writer.cursor.position();
		writer.write_u32(0)?; // Size placeholder
		writer.write_object_data(&obj.1)?;
		let end = writer.cursor.position();
		let size = (end - start) as u32; // Make sure to include self (don't subtract 4)
		writer.cursor.set_position(start);
		writer.write_u32(size)?;
		writer.cursor.set_position(end);
	}

	Ok(writer.cursor.into_inner())
}
