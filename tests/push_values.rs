#[cfg(test)]
mod test {
    use rust_toylock::ticket_lock;
	use std::thread;

	#[test]
	fn push_u32_lock() {
		let test: &'static ticket_lock::TicketLock<Vec<i32>> = Box::leak(Box::new(ticket_lock::TicketLock::new(vec![])));

		let _joins: Vec<_> = (1..=1000)
			.map(|x| {
				thread::spawn(move || {
					let mut a = test.lock();
					a.push(x);
					a.unlock();	
				})
				.join()
				.unwrap();
		}).collect();
	}
}



