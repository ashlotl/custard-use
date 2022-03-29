/// Keeps track of whether an `OCTask`'s `Fulfiller` has a completed prerequisite (parent).
///
/// In order to keep track of whether something is ready, we *do not* use atomics. Instead we just keep track of progress with a moving iterator.
///
/// The pattern is as follows:
///
/// |node A (entrypoint)|	node B|	node C|	whatever happened|
/// |----|----|----|----|
/// |0|0|0|(default state)|
/// |1|0|0|(node A completes, because it is an entrypoint, and sets itself to be one higher than its greatest prerequisite)|
/// |1|2|0|	(node B completes, because it's prerequisite is greater than it. It follows A's suit)|
/// |1|2|3|	(node C completes, for similar reasons)|
/// |4|						2|			3|		(node A completes because C was higher)|
/// |4|						5|			3|		(and so forth. u64 counters are big.)|
///
#[derive(Debug)]
pub struct Ready {
	//TODO: atomic are actually necessary here cuz registers and whatnot
	//careful auto-implementing traits here. This whole struct is designed to be used in a race condition.
	state: u64,
	greatest_prereq: u64,
	entrypoint: bool,
}

impl Ready {
	pub(crate) fn new(entrypoint: bool) -> Self {
		Self { state: 0, greatest_prereq: 0, entrypoint }
	}

	pub(crate) fn release(&self) {
		unsafe {
			(*(self as *const Self as *mut Self)).state = self.greatest_prereq + 1;
		}
	}

	pub(crate) fn load_prerequisite(&self, other: &Self) -> bool {
		let ostate = other.state;
		unsafe {
			(*(self as *const Self as *mut Self)).greatest_prereq = ostate.max(self.greatest_prereq);
		}
		ostate > self.state || self.entrypoint && self.state == 0 || other as *const Self == self as *const Self
	}
}

#[cfg(test)]
mod tests {
	//TODO: eewwwwww

	use super::Ready;

	use std::{
		sync::{
			atomic::{AtomicU32, AtomicUsize, Ordering},
			Arc,
		},
		thread,
	};

	#[test]
	fn check_for_deadlocks() -> Result<(), String> {
		const LOOP_SIZE: usize = 5; //constraint that you *cannot* make a loop of size 1
		const STOP_AT: usize = 1000;

		let mut joins = vec![];

		let count = Arc::new(AtomicUsize::new(0));

		let readys: Arc<Vec<Ready>> = Arc::new((0..LOOP_SIZE).map(|i| Ready::new(i == 0)).collect());

		for i in 0..LOOP_SIZE {
			let count = count.clone();
			let readys = readys.clone();
			joins.push(thread::spawn(move || loop {
				loop {
					//wait via somewhat gross spinning technique
					if readys[i].load_prerequisite(&readys[((i as i32 - 1 + readys.len() as i32) as usize % readys.len())]) {
						break;
					}
					thread::yield_now();
				}

				readys[i].release(); // let other nodes run

				if count.fetch_add(1, Ordering::Relaxed) >= STOP_AT + 1 {
					return;
				}
			}));
		}

		for join in joins {
			join.join().unwrap();
		}

		Ok(())
	}

	#[test]
	fn check_for_race_conditions() -> Result<(), String> {
		const RACE_CONDITION_CHECKS: usize = 10;
		for _i in 0..RACE_CONDITION_CHECKS {
			attempt_to_create_race_condition()?;
		}
		Ok(())
	}

	fn attempt_to_create_race_condition() -> Result<(), String> {
		const NUM_SPLITS: u32 = 6;
		const SPLITS_PER: u32 = 3;

		let num_nodes = nodes_in_tree(NUM_SPLITS, SPLITS_PER);

		let record = Arc::new(vec![(-1, -1); num_nodes]);
		let readys = Arc::new(generate_readys(NUM_SPLITS, SPLITS_PER));

		let mut joins = vec![];

		let count = Arc::new(AtomicU32::new(0));

		for i in 0..readys.len() {
			for j in 0..readys[i].len() {
				let count = count.clone();
				let record = record.clone();
				let readys = readys.clone();

				joins.push(thread::spawn(move || {
					let prerequisites: Vec<&Ready> = (0..SPLITS_PER)
						.map(|k| {
							let other_i = ((i as i32 - 1 + readys.len() as i32) % readys.len() as i32) as usize;
							let layer = &readys[other_i];
							let other_j = (j as u32 * SPLITS_PER + k) as usize % layer.len();
							if layer.len() != SPLITS_PER as usize {
								assert_eq!(layer.len() / SPLITS_PER as usize, readys[i].len());
							}
							&layer[other_j]
						})
						.collect();

					'outer: loop {
						//really ugly busy wait
						for prerequisite in &prerequisites {
							if !readys[i][j].load_prerequisite(prerequisite) {
								continue 'outer;
							}
						}
						break;
					}

					unsafe {
						(*((&*record) as *const Vec<(i32, i32)> as *mut Vec<(i32, i32)>))[count.fetch_add(1, Ordering::Relaxed) as usize] = (i as i32, j as i32);
					} //keep track of the order in which things were executed

					readys[i][j].release(); //release for next nodes
				}));
			}
		}

		for join in joins {
			join.join().unwrap();
		}

		check_records(&record, SPLITS_PER as usize)?;

		Ok(())
	}

	fn check_records(record: &Vec<(i32, i32)>, splits_per: usize) -> Result<(), String> {
		for i in 0..record.len() {
			if record[i].0 == 0 {
				continue;
			}
			for j in 0..splits_per {
				if !record[0..i].contains(&(record[i].0 - 1, record[i].1 * splits_per as i32 + j as i32)) {
					return Err(format!("Ordering failed: {:?} has no ancestor", record[i]));
				}
			}
		}
		Ok(())
	}

	fn generate_readys(num_splits: u32, splits_per: u32) -> Vec<Vec<Ready>> {
		let mut readys = vec![];
		for i in 0..num_splits {
			let mut layer = vec![];
			for _j in 0..splits_per.pow(num_splits - i) {
				layer.push(Ready::new(i == 0));
			}
			readys.push(layer);
		}
		readys
	}

	fn nodes_in_tree(num_splits: u32, splits_per: u32) -> usize {
		let mut ret = 0;
		for p in 1..=num_splits {
			ret += splits_per.pow(p);
		}
		ret as usize
	}
}
