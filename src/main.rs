use std::fs::File;
use std::fs;
use std::io::{self, BufRead, stdin, stdout, Write, Stdout};
use std::path::Path;
use std::process::Output;
use console::Term;
use clap::{Arg, App};
use std::{time::Duration, thread};
use hotwatch::{EventKind, Event, Hotwatch};

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::color;

enum  OutType {
    Error,
    Fail,
    Normal,
}

fn main() {
    // 解析命令行
    // let matches = App::new("catlog")
    // .version("0.0.1")
    // .author("xiepengfei <xiepengfei@uniontech.com>")
    // .about("Log Viewer")
    // .arg(Arg::with_name("file")
    //          .short("f")
    //          .long("file")
    //          .takes_value(true)
    //          .help("log file"))
    // .arg(Arg::with_name("num")
    //          .short("n")
    //          .long("number")
    //          .takes_value(true)
    //          .help("Five less than your favorite number"))
    // .get_matches();

    // let myfile = matches.value_of("file").unwrap_or("input.txt");

    let flashback: bool = false;
    // 获取终端宽高
    let mut term = Term::stdout();
    let (term_h, term_w) = term.size();
    let mut height = term_h as usize;
    let width = term_w as usize;

    let path = Path::new("/home/uos/.cache/deepin/deepin-movie/deepin-movie.log");
    // let path = Path::new(myfile);

    // 获取文件大小
    // let len = fs::metadata(&path).unwrap().len();
    // println!("len: {}", len);

    let mut stdout = stdout().into_raw_mode().unwrap();
    let text = fs::read_to_string(&path).unwrap();
    let mut lines: Vec<&str> = text.split("\n").collect();
    let mut number = 0;

    if flashback {
        lines.reverse();
    }

    find_next(&"error", &mut 0, &mut lines);

    // thread::spawn(|| {
    //     get_file_update(path);
    // });

    for line in &lines {
        if number == height {
            // number = 0;
            break;
        };
        // writeln!(stdout, "{}", line).unwrap();
        number+=1;
        let mut out_type: OutType = OutType::Normal;
        out_type = from_str_get_color(line);
        match out_type {
            OutType::Error => writeln!(stdout, "{}{}{}", color::Fg(color::Red), line, termion::cursor::Goto(0, number as u16)).unwrap(),
            OutType::Fail => writeln!(stdout, "{}{}{}", color::Fg(color::Yellow), line, termion::cursor::Goto(0, number as u16)).unwrap(),
            _ => writeln!(stdout, "{}{}{}", color::Fg(color::White), line, termion::cursor::Goto(0, number as u16)).unwrap(),
        }
        // stdout.flush().unwrap();
    };
    
    // 终端监听键盘事件
    let stdin = stdin();

    //detecting keydown events
    for c in stdin.keys() {
        //i reckon this speaks for itself
        match c.unwrap() {
            // Key::Char('n') => writeln!(stdout, "{}{}", termion::clear::All, 1),
            Key::Ctrl('h') => println!("Hello world!"),
            Key::Char('n') => number = write_next_page(&mut lines, &mut number, &mut term),
            Key::Char('b') => number = write_previous_page(&mut lines, &mut number, &mut term),
            Key::Char('e') => number = find_next(&"error", &mut number, &mut lines),
            Key::Char('q') => break,
            _ => (),
        }

        stdout.flush().unwrap();
    }
    
    // // 在生成输出之前，文件主机必须存在于当前路径中
    // if let Ok(lines) = read_lines(&path) {
    //     // 使用迭代器，返回一个（可选）字符串
    //     for line in lines {
    //         if let Ok(ip) = line {
    //             println!("{}", ip);
    //         }      
    //     }   
    // }
}

// 输出包裹在 Result 中以允许匹配错误，
// 将迭代器返回给文件行的读取器（Reader）。
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn write_next_page(lines: &mut Vec<&str>, current_index: &mut usize, term: &mut Term) -> usize{
    let mut stdout = stdout().into_raw_mode().unwrap();
    let mut write_number = current_index.clone();
    let (term_h, term_w) = term.size();
    let height = term_h as usize;
    let mut current_index: usize = 0;
    writeln!(stdout, "{}{}", termion::cursor::Goto(0, 1), termion::clear::All).unwrap();
    while current_index <= height {
        let current_line = write_number;
        write_number+=1;
        current_index+=1;
        let line = lines[current_line];
        let mut out_type: OutType = OutType::Normal;
        out_type = from_str_get_color(&line);
        match out_type {
            OutType::Error => writeln!(stdout, "{}{}{}", color::Fg(color::Red), lines[current_line], termion::cursor::Goto(0, write_number as u16)).unwrap(),
            OutType::Fail => writeln!(stdout, "{}{}{}", color::Fg(color::Yellow), lines[current_line], termion::cursor::Goto(0, write_number as u16)).unwrap(),
            _ => writeln!(stdout, "{}{}{}", color::Fg(color::White), lines[current_line], termion::cursor::Goto(0, write_number as u16)).unwrap(),
        }
    }
    return write_number;
}

fn write_previous_page(lines: &mut Vec<&str>, current_index: &mut usize, term: &mut Term) -> usize {
    let mut stdout = stdout().into_raw_mode().unwrap();
    let (term_h, term_w) = term.size();
    let height = term_h as usize;
    let mut write_number = current_index.clone() - (height * 2 + 1);
    
    let index = write_next_page(lines, &mut write_number, term);
    return index;
}

fn from_str_get_color(s: &&str) -> OutType{
    let mut my_type = OutType::Normal;
    if s.to_ascii_lowercase().contains("error") {
        my_type = OutType::Error;
    } else if s.to_ascii_lowercase().contains("fail") {
        my_type = OutType::Fail;
    };
    return my_type;
}

fn get_file_update(path: &Path) {

    let mut hotwatch = Hotwatch::new().expect("hotwatch failed to initialize!");
    let mut len = fs::metadata(path).unwrap().len();
    hotwatch
        .watch(path, |event: Event| {
            if let EventKind::Modify(_) = event.kind {
                // len = fs::metadata(path).unwrap().len();
                println!("War has changed{:?}.", event.kind);
            }
        })
        .expect("failed to watch file!");
 
    loop {
        thread::sleep(Duration::from_secs(2));
    }
}

fn find_next(s: &&str, current_index: &mut usize, lines: &mut Vec<&str>) -> usize{
    let mut index:i32 = -1;
    for (i, _line) in lines.iter().enumerate().filter(|x| (x.0 > *current_index)) {
        println!("{}", _line);
        if _line.to_ascii_lowercase().contains(s) {
            index = i as i32;
            break;
        }
    }
    if index > 0 {
        let mut term = Term::stdout();
        let (term_h, term_w) = term.size();
        let mut height = term_h as usize;

        index = index - height as i32 / 2;
        if index < 0 {
            index = 0;
        }

        let write_index: usize = write_next_page(lines, &mut (index as usize), &mut Term::stdout());
        return write_index;
    } else {
        return *current_index;
    }
}

fn find_previous(s: &&str, current_index: &mut usize, lines: &mut Vec<&str>) {

}