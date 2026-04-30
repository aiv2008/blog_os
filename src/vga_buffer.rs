use core::fmt::{self, Write};
use volatile::Volatile;
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((foreground as u8) << 4 | (background as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

// error[E0015]: calls in statics are limited to constant functions, tuple structs and tuple variants
//  --> src/vga_buffer.rs:7:17
//   |
// 7 |     color_code: ColorCode::new(Color::Yellow, Color::Black),
//   |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

// error[E0396]: raw pointers cannot be dereferenced in statics
//  --> src/vga_buffer.rs:8:22
//   |
// 8 |     buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
//   |                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ dereference of raw pointer in constant

// error[E0017]: references in statics may only refer to immutable values
//  --> src/vga_buffer.rs:8:22
//   |
// 8 |     buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
//   |                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ statics require immutable values

// error[E0017]: references in statics may only refer to immutable values
//  --> src/vga_buffer.rs:8:13
//   |
// 8 |     buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
//   |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ statics require immutable values
// 为了明白现在发生了什么，我们需要知道一点：一般的变量在运行时初始化，而静态变量在编译时初始化。
// Rust编译器规定了一个称为常量求值器（const evaluator）的组件，它应该在编译时处理这样的初始化工作。
// 虽然它目前的功能较为有限，但对它的扩展工作进展活跃，比如允许在常量中 panic 的一篇 RFC 文档。
// 关于 ColorCode::new 的问题应该能使用常函数（const functions）解决，但常量求值器还存在不完善之处，它还不能在编译时直接转换裸指针到变量的引用——也许未来这段代码能够工作，但在那之前，我们需要寻找另外的解决方案。
// 延迟初始化
// 使用非常函数初始化静态变量是 Rust 程序员普遍遇到的问题。幸运的是，有一个叫做 lazy_static 的包提供了一个很棒的解决方案：它提供了名为 lazy_static! 的宏，定义了一个延迟初始化（lazily initialized）的静态变量；这个变量的值将在第一次使用时计算，而非在编译时计算。这时，变量的初始化过程将在运行时执行，任意的初始化代码——无论简单或复杂——都是能够使用的。
pub static WRITER: Writer = Writer{
    column_position: 0,
    color_code: ColorCode::new(Color::Blue, Color::LightRed),
    buffer: unsafe {
        &mut *(0xb8000 as *mut Buffer)
    },
}

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

// column_position: 跟踪光标在最后一行的位置
// color_code： 指定当前字符的前景和背景色
// buffer： VGA 字符缓冲区的可变借用
pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => {
                self.new_line();
            }
            _byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
                // 新写入的字符永远都在倒数第一行，所以是BUFFER_HEIGHT - 1
                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;
                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: _byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => {
                    self.write_byte(byte);
                }
                _ => self.write_byte(0xfe),
            }
        }
    }
    // 我们遍历每个屏幕上的字符，把每个字符移动到它上方一行的相应位置。
    // 这里，.. 符号是区间标号（range notation）的一种；它表示左闭右开的区间，因此不包含它的上界。
    // 在外层的枚举中，我们从第 1 行开始，省略了对第 0 行的枚举过程——因为这一行应该被移出屏幕，即它将被下一行的字符覆写。
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    pub fn print_sth() {
        let mut writer = Writer {
            column_position: 0,
            color_code: ColorCode::new(Color::Blue, Color::LightRed),
            buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
        };

        writer.write_byte(b'H');
        writer.write_string("ello ");
        writer.write_string("Wörld!\n");
        write!(writer, "the number are {} and {}", 34, 1.0 / 3.2).unwrap();
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}
