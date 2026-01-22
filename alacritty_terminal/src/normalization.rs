use unicode_normalization::{char::is_combining_mark, UnicodeNormalization};
use vte::ansi::{self, Handler};

pub struct NormalizationHandler<'a, T> {
    term: &'a mut T,
    buffer: &'a mut String,
    needs_normalization: bool,
}

impl<'a, T: Handler> NormalizationHandler<'a, T> {
    pub fn new(term: &'a mut T, buffer: &'a mut String) -> Self {
        Self { term, buffer, needs_normalization: false }
    }

    pub fn flush(&mut self) {
        if self.buffer.is_empty() {
            return;
        }

        if self.needs_normalization {
            for c in self.buffer.nfc() {
                self.term.input(c);
            }
        } else {
            for c in self.buffer.chars() {
                self.term.input(c);
            }
        }

        self.buffer.clear();
        self.needs_normalization = false;
    }

    #[inline]
    fn mark_if_needs_normalization(&mut self, c: char) {
        if self.needs_normalization {
            return;
        }

        if is_combining_mark(c) || is_hangul_jamo(c) {
            self.needs_normalization = true;
        }
    }
}

#[inline]
fn is_hangul_jamo(c: char) -> bool {
    matches!(
        c as u32,
        0x1100..=0x11FF | 0xA960..=0xA97F | 0xD7B0..=0xD7FF
    )
}

impl<'a, T: Handler> Handler for NormalizationHandler<'a, T> {
    #[inline]
    fn input(&mut self, c: char) {
        self.buffer.push(c);
        self.mark_if_needs_normalization(c);
    }

    #[inline]
    fn goto(&mut self, line: i32, col: usize) {
        self.flush();
        self.term.goto(line, col);
    }

    #[inline]
    fn goto_line(&mut self, line: i32) {
        self.flush();
        self.term.goto_line(line);
    }

    #[inline]
    fn goto_col(&mut self, col: usize) {
        self.flush();
        self.term.goto_col(col);
    }

    #[inline]
    fn decaln(&mut self) {
        self.flush();
        self.term.decaln();
    }

    #[inline]
    fn move_up(&mut self, lines: usize) {
        self.flush();
        self.term.move_up(lines);
    }

    #[inline]
    fn move_down(&mut self, lines: usize) {
        self.flush();
        self.term.move_down(lines);
    }

    #[inline]
    fn move_forward(&mut self, cols: usize) {
        self.flush();
        self.term.move_forward(cols);
    }

    #[inline]
    fn move_backward(&mut self, cols: usize) {
        self.flush();
        self.term.move_backward(cols);
    }

    #[inline]
    fn identify_terminal(&mut self, intermediate: Option<char>) {
        self.flush();
        self.term.identify_terminal(intermediate);
    }

    #[inline]
    fn device_status(&mut self, arg: usize) {
        self.flush();
        self.term.device_status(arg);
    }

    #[inline]
    fn move_down_and_cr(&mut self, lines: usize) {
        self.flush();
        self.term.move_down_and_cr(lines);
    }

    #[inline]
    fn move_up_and_cr(&mut self, lines: usize) {
        self.flush();
        self.term.move_up_and_cr(lines);
    }

    #[inline]
    fn put_tab(&mut self, count: u16) {
        self.flush();
        self.term.put_tab(count);
    }

    #[inline]
    fn backspace(&mut self) {
        self.flush();
        self.term.backspace();
    }

    #[inline]
    fn carriage_return(&mut self) {
        self.flush();
        self.term.carriage_return();
    }

    #[inline]
    fn linefeed(&mut self) {
        self.flush();
        self.term.linefeed();
    }

    #[inline]
    fn bell(&mut self) {
        self.flush();
        self.term.bell();
    }

    #[inline]
    fn substitute(&mut self) {
        self.flush();
        self.term.substitute();
    }

    #[inline]
    fn newline(&mut self) {
        self.flush();
        self.term.newline();
    }

    #[inline]
    fn set_horizontal_tabstop(&mut self) {
        self.flush();
        self.term.set_horizontal_tabstop();
    }

    #[inline]
    fn scroll_up(&mut self, lines: usize) {
        self.flush();
        self.term.scroll_up(lines);
    }

    #[inline]
    fn scroll_down(&mut self, lines: usize) {
        self.flush();
        self.term.scroll_down(lines);
    }

    #[inline]
    fn insert_blank_lines(&mut self, lines: usize) {
        self.flush();
        self.term.insert_blank_lines(lines);
    }

    #[inline]
    fn delete_lines(&mut self, lines: usize) {
        self.flush();
        self.term.delete_lines(lines);
    }

    #[inline]
    fn erase_chars(&mut self, count: usize) {
        self.flush();
        self.term.erase_chars(count);
    }

    #[inline]
    fn delete_chars(&mut self, count: usize) {
        self.flush();
        self.term.delete_chars(count);
    }

    #[inline]
    fn move_backward_tabs(&mut self, count: u16) {
        self.flush();
        self.term.move_backward_tabs(count);
    }

    #[inline]
    fn move_forward_tabs(&mut self, count: u16) {
        self.flush();
        self.term.move_forward_tabs(count);
    }

    #[inline]
    fn save_cursor_position(&mut self) {
        self.flush();
        self.term.save_cursor_position();
    }

    #[inline]
    fn restore_cursor_position(&mut self) {
        self.flush();
        self.term.restore_cursor_position();
    }

    #[inline]
    fn clear_line(&mut self, mode: ansi::LineClearMode) {
        self.flush();
        self.term.clear_line(mode);
    }

    #[inline]
    fn set_color(&mut self, index: usize, color: ansi::Rgb) {
        self.flush();
        self.term.set_color(index, color);
    }

    #[inline]
    fn dynamic_color_sequence(&mut self, prefix: String, index: usize, terminator: &str) {
        self.flush();
        self.term.dynamic_color_sequence(prefix, index, terminator);
    }

    #[inline]
    fn reset_color(&mut self, index: usize) {
        self.flush();
        self.term.reset_color(index);
    }

    #[inline]
    fn clipboard_store(&mut self, clipboard: u8, base64: &[u8]) {
        self.flush();
        self.term.clipboard_store(clipboard, base64);
    }

    #[inline]
    fn clipboard_load(&mut self, clipboard: u8, terminator: &str) {
        self.flush();
        self.term.clipboard_load(clipboard, terminator);
    }

    #[inline]
    fn clear_screen(&mut self, mode: ansi::ClearMode) {
        self.flush();
        self.term.clear_screen(mode);
    }

    #[inline]
    fn clear_tabs(&mut self, mode: ansi::TabulationClearMode) {
        self.flush();
        self.term.clear_tabs(mode);
    }

    #[inline]
    fn reset_state(&mut self) {
        self.flush();
        self.term.reset_state();
    }

    #[inline]
    fn reverse_index(&mut self) {
        self.flush();
        self.term.reverse_index();
    }

    #[inline]
    fn set_hyperlink(&mut self, hyperlink: Option<ansi::Hyperlink>) {
        self.flush();
        self.term.set_hyperlink(hyperlink);
    }

    #[inline]
    fn terminal_attribute(&mut self, attr: ansi::Attr) {
        self.flush();
        self.term.terminal_attribute(attr);
    }

    #[inline]
    fn set_private_mode(&mut self, mode: ansi::PrivateMode) {
        self.flush();
        self.term.set_private_mode(mode);
    }

    #[inline]
    fn unset_private_mode(&mut self, mode: ansi::PrivateMode) {
        self.flush();
        self.term.unset_private_mode(mode);
    }

    #[inline]
    fn report_private_mode(&mut self, mode: ansi::PrivateMode) {
        self.flush();
        self.term.report_private_mode(mode);
    }

    #[inline]
    fn set_mode(&mut self, mode: ansi::Mode) {
        self.flush();
        self.term.set_mode(mode);
    }

    #[inline]
    fn unset_mode(&mut self, mode: ansi::Mode) {
        self.flush();
        self.term.unset_mode(mode);
    }

    #[inline]
    fn report_mode(&mut self, mode: ansi::Mode) {
        self.flush();
        self.term.report_mode(mode);
    }

    #[inline]
    fn set_scrolling_region(&mut self, top: usize, bottom: Option<usize>) {
        self.flush();
        self.term.set_scrolling_region(top, bottom);
    }

    #[inline]
    fn set_keypad_application_mode(&mut self) {
        self.flush();
        self.term.set_keypad_application_mode();
    }

    #[inline]
    fn unset_keypad_application_mode(&mut self) {
        self.flush();
        self.term.unset_keypad_application_mode();
    }

    #[inline]
    fn configure_charset(&mut self, index: ansi::CharsetIndex, charset: ansi::StandardCharset) {
        self.flush();
        self.term.configure_charset(index, charset);
    }

    #[inline]
    fn set_active_charset(&mut self, index: ansi::CharsetIndex) {
        self.flush();
        self.term.set_active_charset(index);
    }

    #[inline]
    fn set_title(&mut self, title: Option<String>) {
        self.flush();
        self.term.set_title(title);
    }

    #[inline]
    fn push_title(&mut self) {
        self.flush();
        self.term.push_title();
    }

    #[inline]
    fn pop_title(&mut self) {
        self.flush();
        self.term.pop_title();
    }

    #[inline]
    fn text_area_size_pixels(&mut self) {
        self.flush();
        self.term.text_area_size_pixels();
    }

    #[inline]
    fn text_area_size_chars(&mut self) {
        self.flush();
        self.term.text_area_size_chars();
    }

    #[inline]
    fn set_cursor_style(&mut self, style: Option<ansi::CursorStyle>) {
        self.flush();
        self.term.set_cursor_style(style);
    }

    #[inline]
    fn set_cursor_shape(&mut self, shape: ansi::CursorShape) {
        self.flush();
        self.term.set_cursor_shape(shape);
    }

    #[inline]
    fn report_keyboard_mode(&mut self) {
        self.flush();
        self.term.report_keyboard_mode();
    }

    #[inline]
    fn push_keyboard_mode(&mut self, mode: ansi::KeyboardModes) {
        self.flush();
        self.term.push_keyboard_mode(mode);
    }

    #[inline]
    fn pop_keyboard_modes(&mut self, to_pop: u16) {
        self.flush();
        self.term.pop_keyboard_modes(to_pop);
    }

    #[inline]
    fn set_keyboard_mode(
        &mut self,
        mode: ansi::KeyboardModes,
        behavior: ansi::KeyboardModesApplyBehavior,
    ) {
        self.flush();
        self.term.set_keyboard_mode(mode, behavior);
    }

    #[inline]
    fn insert_blank(&mut self, count: usize) {
        self.flush();
        self.term.insert_blank(count);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::{Column, Line};
    use crate::term::test::mock_term;
    use vte::ansi::Handler;

    #[test]
    fn normalization_works() {
        let mut term = mock_term("  "); // Dummy term with 2 columns
        let mut buffer = String::new();
        let mut handler = NormalizationHandler::new(&mut term, &mut buffer);

        // Hangul Jamo: ᄀ (U+1100) + ᅡ (U+1161) -> 가 (U+AC00)
        handler.input('\u{1100}');
        handler.input('\u{1161}');

        // Before flushed, nothing in grid (buffer holds it)
        handler.flush();

        // Check grid content.
        let cell = &term.grid()[Line(0)][Column(0)];
        assert_eq!(cell.c, '\u{ac00}');
    }
}
