# Qshot

Qshot is a high performance crate that allows you to take screenshots quickly and easily on Windows.

## What exactly is this library?

It's just a thin wrapper around a bunch of winapi functions to make screenshotting of a particular area as fast and easy as possible, while keeping memory safe.

It does not make any assumptions on what you may want to do with the data, you get a raw slice containing bitmap bit values and that's it.

## Example usage

```rust
use std::error::Error;
use qshot::CaptureManager;

fn main() -> Result<(), Box<dyn Error>> {
	let manager = CaptureManager::new(0, (250, 250), (500, 500))?;

	for i in 0..1000 {
		if i == 500 {
			manager.change_size((100, 100), (100, 250));
		}
		let res = manager.capture()?;
		do_something(res.get_bits());
	}
	Ok(())
}
```

## Contribution
Feel free to open a pull request if you think that something could have been done better or more efficiently or at least open an issue so I can look into that.