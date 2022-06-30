#![feature(rustc_private)]
//needs to be compiled with a nightly chain because of this

use std::fs;
use std::io::Write;
use std::path::Path;
use std::env;
use std::os::unix::fs::PermissionsExt;

extern crate libc;

use libc::{c_ulong, c_ushort, STDOUT_FILENO};
use libc::ioctl;

#[repr(C)]
struct winsize {
    ws_row: c_ushort, /* rows, in characters */
    ws_col: c_ushort, /* columns, in characters */
    ws_xpixel: c_ushort, /* horizontal size, pixels */
    ws_ypixel: c_ushort /* vertical size, pixels */
}

const TIOCGWINSZ: c_ulong = 0x5413;

#[derive(Debug)]
enum Etype {
    Dir,
    Exe,
    Txt
}

fn main() {
    let time = std::time::Instant::now();
    // I would make this a constant, but to_string() is forbidden there
    let config_default: String = "truncate:false\ntruncate_at:10\nabsolute_limit:40\nitems:4\nspacing:2".to_string();

    let curr_dir = std::env::current_dir().unwrap();
    let terminal_width = get_winsize().0;

    // flags for cli
    let mut show_hidden = false;
    let mut path = Path::new(&curr_dir);
    let mut use_config = true;
    let mut timed = false;
    
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);
    for i in 0..args.len() {
	match &args[i][..] {
	    "-nc" => use_config = false,
	    "-a" => show_hidden = true,
	    "-t" => timed = true,
	    s => path = Path::new(s)
	}
    }

    let mut truncate_b  = false;
    let mut truncate_at = 10;
    let mut absolute_length_limit = 40;
    let mut spacing = 2;
    let mut items_per_line: usize = 4;
    let mut ipl_configed = false;
    //terminal_width as usize/(absolute_length_limit+spacing)

    if use_config {
	let mut home_dir = env::var("HOME").unwrap();
	home_dir.push_str("/.config/rist");
	let config_path = Path::new(&home_dir);
	if !config_path.is_file() {
	    println!("Config either not a file, or doesn't exist!");
	    println!("Creating config with sensible defaults!");
	    println!("Config resides in `~/.config/rist`!");
	    
	    let mut config_file = fs::File::create(config_path).unwrap();
	    writeln!(&mut config_file, "{}", &config_default).unwrap();
	} else {
	    let config_str = fs::read_to_string(config_path).unwrap();
	    for l in config_str.lines() {
		let mut l = l.to_string();
		if l.starts_with("truncate:") {
		    l.drain(0..9);
		    truncate_b = l.parse().unwrap();
		} else if l.starts_with("truncate_at:") {
		    l.drain(0..12);
		    truncate_at = l.parse().unwrap();
		} else if l.starts_with("absolute_limit:") {
		    l.drain(0..15);
		    absolute_length_limit = l.parse().unwrap();
		} else if l.starts_with("items:") {
		    l.drain(0..6);
		    items_per_line = l.parse().unwrap();
		    ipl_configed = true;
		} else if l.starts_with("spacing:") {
		    l.drain(0..8);
		    spacing = l.parse().unwrap();
		}
	    }
	}
    }
    
    let entries_iterator = fs::read_dir(path).unwrap();
    let mut entries_iterator_types = fs::read_dir(path).unwrap();
    let mut entries_vec: Vec<(String, Etype)> = Vec::new();
    
    for i in entries_iterator {
	let j = entries_iterator_types.next().unwrap().unwrap();
	// get entry type
	let etype: Etype;
	if j.file_type().unwrap().is_dir() {
	    etype = Etype::Dir;
	} else  if j.metadata().unwrap().permissions().mode() & 0o111 != 0{
	    etype = Etype::Exe;
	} else {
	    etype = Etype::Txt;
	}
	// convert entry to string
	let mut entry = i.unwrap().file_name().into_string().unwrap();
	//truncate string
	if truncate_b {
	    entry.truncate(truncate_at);
	} else {
	    entry.truncate(absolute_length_limit);
	}
	//check if file is hidden, also if they're being shown
	if entry.starts_with(".") {
	    if show_hidden {
		entries_vec.push((entry, etype));
	    }
	} else {
	    if !truncate_b {
		if entry.len() > truncate_at {
		    truncate_at = entry.len()
		}
	    }
	    entries_vec.push((entry, etype));
	}
    }

    //pad entries
    for i in 0..entries_vec.len() {
	let mut e = entries_vec.remove(i);
	while e.0.len() < truncate_at {
	    e.0.push(' ');
	}
	entries_vec.insert(i, e);
    }
    if !ipl_configed {
	items_per_line = terminal_width as usize/(truncate_at+spacing);
    }
    
    //print files
    for i in 1..entries_vec.len() {
	if entries_vec[i].0.starts_with('.') {
	    match entries_vec[i].1 {
		Etype::Dir => print!("\x1b[0;34m"),
		Etype::Exe => print!("\x1b[0;33m"),
		Etype::Txt => print!("\x1b[1;30m")
	    }
	} else {
	    match entries_vec[i].1 {
		Etype::Dir => print!("\x1b[1;34m"),
		Etype::Exe => print!("\x1b[1;33m"),
		Etype::Txt => print!("\x1b[0;37m")
	    }
	}
	print!("{}\x1b[0;37m", entries_vec[i].0);
	//spacing between the files
	for _ in 0..spacing {
	    print!(" ");
	}
	if i % items_per_line == 0 {
	    print!("\n");
	}
    }
    print!("\n");
    if timed {
	println!("Took: [{:?}]", time.elapsed());
    }
}

fn get_winsize() -> (isize, isize) {
    let w = winsize { ws_row: 0, ws_col: 0, ws_xpixel: 0, ws_ypixel: 0 };
    let r = unsafe { ioctl(STDOUT_FILENO, TIOCGWINSZ, &w) };

    match r {
        0 => (w.ws_col as isize, w.ws_row as isize),
        _ => {
            return (0, 0)
        }
    }
}
