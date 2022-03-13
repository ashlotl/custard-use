#[derive(Clone, Debug)]
pub enum InstanceControlFlow {
	Continue,
	FullReload,
	PartialReload,
	Stop,
}
