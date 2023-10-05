fn process_vm_readv(pid: u64, addr: u64, size: usize) -> Result<Vec<u8>, String> {
	#[repr(C)]
	struct IoVec {
		addr: *mut u8,
		size: usize,
	}

	let mut buffer = Vec::new();
	buffer.resize(size, 0);

	let local = IoVec {
		addr: buffer.as_mut_ptr(),
		size: buffer.len(),
	};

	let remote = IoVec {
		addr: addr as *mut u8,
		size,
	};

	let mut rc: i64;

	unsafe {
		std::arch::asm!(
			"syscall",
			inlateout("rax") 310u64 => rc,
			inout("rdi") pid => _,
			inout("rsi") &local as *const IoVec as u64 => _,
			inout("rdx") 1 => _,
			inout("r10") &remote as *const IoVec as u64 => _,
			inout("r8") 1 => _,
			inout("r9") 0 => _,
			out("rcx") _,
			out("r11") _,
		);
	}

	if rc == -1 || rc != size as i64 {
		return Err(format!(
			"failed to read from pid: {pid}, address: {addr:x}, size: {size}, rc: {rc}"
		));
	}

	return Ok(buffer);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let args = std::env::args().collect::<Vec<String>>();

	let usage_text = format!(
		"usage: {} <pid> <start> <end>",
		args.get(0).unwrap_or(&String::from("program"))
	);

	let pid = args
		.get(1)
		.ok_or_else(|| usage_text.clone())?
		.parse::<u64>()
		.or(Err("cannot parse 'pid'"))?;

	let start = u64::from_str_radix(args.get(2).ok_or_else(|| usage_text.clone())?, 16)
		.or(Err("cannot parse 'start'"))?;

	let end =
		u64::from_str_radix(args.get(3).ok_or(usage_text)?, 16).or(Err("cannot parse 'end'"))?;

	let bytes = process_vm_readv(pid, start, (end - start) as usize)?;
	std::fs::write("/proc/self/fd/1", bytes)?;

	Ok(())
}
