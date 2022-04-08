use std::{
	collections::BTreeMap,
	fmt::Debug,
	marker::PhantomData,
	ops::{Deref, DerefMut},
};

use crate::{
	composition::loaded::loaded_crate::LoadedCrate,
	concurrency::access::{Access, AccessType},
	identify::{crate_name::CrateName, datachunk_name::FullDatachunkName},
	user_types::datachunk::Datachunkable,
	utils::mutable_arc::MutableArc,
};

#[derive(Clone, Debug)]
pub enum DatachunkGetterResult<T, A>
where
	A: Datachunkable,
	T: Deref<Target = A>,
{
	Ok(T),
	CrateNotFound,
	DatachunkNotInCrate,
	NoImmutableAccessAllowed,
	NoMutableAccessAllowed,
	WrongType,
}

impl<'a, Mutability: Debug, A: Datachunkable>
	DatachunkGetterResult<DatachunkWrapper<Mutability, A>, A>
{
	pub fn or_panic(self) -> DatachunkWrapper<Mutability, A> {
		if let Self::Ok(v) = self {
			return v;
		}
		panic!("Non-Ok value of DatachunkGetterResult: {:?}", self);
	}
}

pub struct DatachunkGetter {
	crate_table: MutableArc<BTreeMap<CrateName, LoadedCrate>>,
	accesses: Vec<Access>,
}

impl DatachunkGetter {
	pub fn new(
		crate_table: MutableArc<BTreeMap<CrateName, LoadedCrate>>,
		accesses: Vec<Access>,
	) -> Self {
		Self {
			crate_table,
			accesses,
		}
	}

	fn check_access(
		&self,
		name: &FullDatachunkName,
		mutable: AccessType,
	) -> bool {
		let mut found = false;

		for access in &self.accesses {
			if &access.of == name
				&& match mutable {
					AccessType::ImmutableAccess => true,
					AccessType::MutableAccess => {
						access.mut_immut == AccessType::MutableAccess
					}
				} {
				found = true;
				break;
			}
		}
		found
	}

	fn get<Mutability, T: Datachunkable>(
		&self,
		name: &FullDatachunkName,
	) -> DatachunkGetterResult<DatachunkWrapper<Mutability, T>, T> {
		println!("crate table: {:#?}", self.crate_table);
		match self.crate_table.get().get(&name.crate_name) {
			Some(loaded_crate) => {
				match loaded_crate.datachunks.get(&name.datachunk_name) {
					Some(loaded_datachunk) => {
						let dyn_object = loaded_datachunk
							.as_ref()
							.unwrap()
							.user_data
							.clone();
						let v = unsafe {
							match (&mut *(dyn_object.get_mut()
								as *mut dyn Datachunkable))
								.downcast_mut::<T>()
							{
								Some(v) => v,
								None => {
									return DatachunkGetterResult::WrongType
								}
							}
						};
						return DatachunkGetterResult::Ok(DatachunkWrapper {
							phantom: PhantomData::default(),
							inner: v,
							_preserve_lifetime: dyn_object,
						});
					}
					None => {
						return DatachunkGetterResult::DatachunkNotInCrate;
					}
				}
			}
			None => {
				return DatachunkGetterResult::CrateNotFound;
			}
		}
	}

	pub fn get_immut<T: Datachunkable>(
		&self,
		name: &FullDatachunkName,
	) -> DatachunkGetterResult<DatachunkWrapper<Immutable, T>, T> {
		if !self.check_access(name, AccessType::ImmutableAccess) {
			return DatachunkGetterResult::NoImmutableAccessAllowed;
		}
		match self.get(name) {
			DatachunkGetterResult::Ok(v) => DatachunkGetterResult::Ok(v),
			DatachunkGetterResult::CrateNotFound => {
				DatachunkGetterResult::CrateNotFound
			}
			DatachunkGetterResult::DatachunkNotInCrate => {
				DatachunkGetterResult::DatachunkNotInCrate
			}
			DatachunkGetterResult::NoImmutableAccessAllowed => {
				DatachunkGetterResult::NoImmutableAccessAllowed
			}
			DatachunkGetterResult::NoMutableAccessAllowed => {
				DatachunkGetterResult::NoMutableAccessAllowed
			}
			DatachunkGetterResult::WrongType => {
				DatachunkGetterResult::WrongType
			}
		}
	}

	pub fn get_mut<T: Datachunkable>(
		&self,
		name: &FullDatachunkName,
	) -> DatachunkGetterResult<DatachunkWrapper<Mutable, T>, T> {
		if !self.check_access(name, AccessType::MutableAccess) {
			return DatachunkGetterResult::NoMutableAccessAllowed;
		}
		self.get(name)
	}
}

#[derive(Debug)]
pub struct Immutable;

#[derive(Debug)]
pub struct Mutable;

#[derive(Debug)]
pub struct DatachunkWrapper<Mutability, T: Datachunkable> {
	phantom: PhantomData<Mutability>,
	inner: &'static mut T,
	_preserve_lifetime: MutableArc<dyn Datachunkable>, //make sure that the underlying data isn't dropped prematurely
}

impl<Mutability, T: Datachunkable> Deref for DatachunkWrapper<Mutability, T> {
	type Target = T;
	fn deref(&self) -> &T {
		self.inner
	}
}

impl<T: Datachunkable> DerefMut for DatachunkWrapper<Mutable, T> {
	fn deref_mut(&mut self) -> &mut T {
		self.inner
	}
}
