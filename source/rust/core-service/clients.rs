use std::libc;
use std::ptr;

use common::dynamics;
use common::vec;


pub struct ClientMap {
	clients: IdMap,
	idPool : IdPool
}

pub struct IdMap {
	cap  : libc::size_t,
	elems: *mut IdMapEntry
}

pub struct IdMapEntry {
	isOccupied: int,
	value     : Client
}

pub struct IdPool {
	cap  : libc::size_t,
	size : libc::size_t,
	elems: *mut libc::size_t
}

struct Client {
	socketFD: libc::c_int,
	id      : libc::size_t,
	ship    : dynamics::Body
}

impl IdPool {
	fn new(capacity: libc::size_t) -> IdPool {
		let idPoolSize =
			capacity * ::std::mem::size_of::<libc::size_t>() as libc::size_t;

		let mut stack = IdPool {
			cap  : capacity,
			size : 0,
			elems: unsafe {
				libc::malloc(idPoolSize) as *mut libc::size_t } };

		while stack.size < capacity {
			unsafe {
				let ptr = ptr::mut_offset(stack.elems, stack.size as int);
				*ptr = (capacity - stack.size - 1) as libc::size_t; };

			stack.size += 1;
		}

		stack
	}

	fn is_full(&self) -> bool {
		self.size == 0
	}

	fn push(&mut self, id: libc::size_t) {
		unsafe {
			let idPtr = ptr::mut_offset(self.elems, self.size as int);
			(*idPtr) = id; };
		self.size += 1;
	}

	fn pop(&mut self) -> libc::size_t {
		let element = unsafe {
			let ptr = ptr::mut_offset(self.elems, (self.size - 1) as int);
			*ptr };
		self.size -= 1;

		element
	}
}


pub fn new_client_map(cap: libc::size_t) -> ~ClientMap {
	let mut c = ClientMap {
		clients: IdMap {
			cap  : 0,
			elems: ::std::ptr::null::<IdMapEntry>() as *mut IdMapEntry },
		idPool: IdPool::new(cap) };

	// Init IdMap
	c.clients.cap = cap;
	let memSize = cap * ::std::mem::size_of::<IdMapEntry>() as libc::size_t;
	c.clients.elems = unsafe { libc::malloc(memSize) as *mut IdMapEntry };
	unsafe { ptr::set_memory(c.clients.elems, 0, cap as uint) };

	~c
}

pub fn can_add(c: &ClientMap) -> bool {
	!c.idPool.is_full()
}

pub fn add(c: &mut ClientMap, socketFD: libc::c_int, pos: vec::Vec2, vel: vec::Vec2) {
	// Get id from pool.
	let clientId = c.idPool.pop();

	// Construct client
	let client = Client {
		socketFD: socketFD,
		id      : clientId,
		ship    : dynamics::Body { pos: pos, vel: vel } };

	// Add client to map
	unsafe {
		let ptr = ptr::mut_offset(c.clients.elems, clientId as int);
		(*ptr).isOccupied = 1;
		(*ptr).value = client;
	};
}

pub fn remove(c: &mut ClientMap, id: libc::size_t) {
	unsafe {
		let clientPtr = ptr::mut_offset(c.clients.elems, id as int);
		let containsClient = (*clientPtr).isOccupied == 1;

		if containsClient {
			// Remove client
			(*clientPtr).isOccupied = 0;

			c.idPool.push(id);
		}
	}
}
