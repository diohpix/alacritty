/// APC (Application Program Command) filter for Kitty Graphics Protocol.
///
/// Scans raw PTY bytes for `ESC _ G ... ESC \` sequences before vte processes them.
/// Extracts complete Kitty graphics sequences and passes all other bytes through unchanged.

const ESC: u8 = 0x1B;
const UNDERSCORE: u8 = b'_';
const BACKSLASH: u8 = b'\\';
const G: u8 = b'G';

#[derive(Debug, Clone, Copy, PartialEq)]
enum State {
    /// Normal pass-through mode.
    Ground,
    /// Saw ESC (0x1B), waiting to see if next byte is `_`.
    EscSeen,
    /// Inside `ESC _`, waiting to see if next byte is `G` (Kitty graphics).
    ApcStart,
    /// Inside a Kitty Graphics APC: `ESC _ G ...`, accumulating until `ESC \`.
    InKittyApc,
    /// Inside the APC, saw ESC, waiting for `\` to complete the sequence.
    ApcEscSeen,
}

/// Filters Kitty Graphics APC sequences from a PTY byte stream.
///
/// Maintains state across calls to handle sequences split across read boundaries.
pub struct ApcFilter {
    state: State,
    /// Accumulator for current APC body (between `G` and `ESC \`).
    buffer: Vec<u8>,
    /// Completed APC sequence bodies ready for consumption.
    completed: Vec<Vec<u8>>,
    /// Bytes that should be passed through to the vte parser.
    passthrough: Vec<u8>,
}

impl Default for ApcFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl ApcFilter {
    pub fn new() -> Self {
        Self {
            state: State::Ground,
            buffer: Vec::with_capacity(4096),
            completed: Vec::new(),
            passthrough: Vec::with_capacity(4096),
        }
    }

    /// Filter input bytes, extracting Kitty Graphics APC sequences.
    ///
    /// Returns `(passthrough_bytes, completed_apc_bodies)`:
    /// - `passthrough_bytes`: bytes that should be fed to the vte parser
    /// - `completed_apc_bodies`: complete Kitty graphics command bodies (without ESC_G prefix and ESC\ suffix)
    pub fn filter(&mut self, input: &[u8]) -> (&[u8], Vec<Vec<u8>>) {
        self.passthrough.clear();
        self.completed.clear();

        for &byte in input {
            match self.state {
                State::Ground => {
                    if byte == ESC {
                        self.state = State::EscSeen;
                    } else {
                        self.passthrough.push(byte);
                    }
                }

                State::EscSeen => {
                    if byte == UNDERSCORE {
                        // ESC _ — APC introducer, check if it's Kitty graphics
                        self.state = State::ApcStart;
                    } else {
                        // Not an APC, pass through the deferred ESC + this byte
                        self.passthrough.push(ESC);
                        if byte == ESC {
                            // Another ESC — stay in EscSeen
                            self.state = State::EscSeen;
                        } else {
                            self.passthrough.push(byte);
                            self.state = State::Ground;
                        }
                    }
                }

                State::ApcStart => {
                    if byte == G {
                        // ESC _ G — this is a Kitty graphics sequence
                        self.buffer.clear();
                        self.state = State::InKittyApc;
                    } else {
                        // Not a Kitty APC. Pass `ESC _ <byte>` through to vte
                        // (vte will enter SosPmApcString state and discard it)
                        self.passthrough.push(ESC);
                        self.passthrough.push(UNDERSCORE);
                        if byte == ESC {
                            self.state = State::EscSeen;
                        } else {
                            self.passthrough.push(byte);
                            self.state = State::Ground;
                        }
                    }
                }

                State::InKittyApc => {
                    if byte == ESC {
                        self.state = State::ApcEscSeen;
                    } else {
                        self.buffer.push(byte);
                    }
                }

                State::ApcEscSeen => {
                    if byte == BACKSLASH {
                        // ESC \ — String Terminator, APC is complete
                        let body = std::mem::take(&mut self.buffer);
                        self.completed.push(body);
                        self.state = State::Ground;
                    } else if byte == ESC {
                        // Another ESC inside APC (unusual but handle it)
                        self.buffer.push(ESC);
                        // Stay in ApcEscSeen
                    } else {
                        // False alarm — push the deferred ESC and this byte to buffer
                        self.buffer.push(ESC);
                        self.buffer.push(byte);
                        self.state = State::InKittyApc;
                    }
                }
            }
        }

        let completed = std::mem::take(&mut self.completed);
        (&self.passthrough, completed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_apc_passthrough() {
        let mut filter = ApcFilter::new();
        let input = b"hello world\x1b[31m";
        let (pass, completed) = filter.filter(input);
        assert_eq!(pass, input);
        assert!(completed.is_empty());
    }

    #[test]
    fn test_simple_kitty_apc() {
        let mut filter = ApcFilter::new();
        let input = b"\x1b_Ga=t,f=100;AAAA\x1b\\";
        let (pass, completed) = filter.filter(input);
        assert!(pass.is_empty());
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0], b"a=t,f=100;AAAA");
    }

    #[test]
    fn test_mixed_content() {
        let mut filter = ApcFilter::new();
        let input = b"before\x1b_Ga=t;data\x1b\\after";
        let (pass, completed) = filter.filter(input);
        assert_eq!(pass, b"beforeafter");
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0], b"a=t;data");
    }

    #[test]
    fn test_split_across_reads() {
        let mut filter = ApcFilter::new();

        // First chunk: partial APC
        let (pass1, completed1) = filter.filter(b"hello\x1b_Ga=t;da");
        assert_eq!(pass1, b"hello");
        assert!(completed1.is_empty());

        // Second chunk: rest of APC
        let (pass2, completed2) = filter.filter(b"ta\x1b\\world");
        assert_eq!(pass2, b"world");
        assert_eq!(completed2.len(), 1);
        assert_eq!(completed2[0], b"a=t;data");
    }

    #[test]
    fn test_non_kitty_apc() {
        let mut filter = ApcFilter::new();
        // ESC _ X ... (not 'G', so not Kitty)
        let input = b"\x1b_Xsome data\x1b\\";
        let (pass, completed) = filter.filter(input);
        // Should pass through ESC _ X and the rest
        assert!(pass.starts_with(&[ESC, UNDERSCORE, b'X']));
        assert!(completed.is_empty());
    }

    #[test]
    fn test_multiple_apc_sequences() {
        let mut filter = ApcFilter::new();
        let input = b"\x1b_Ga=t;img1\x1b\\\x1b_Ga=p;img2\x1b\\";
        let (pass, completed) = filter.filter(input);
        assert!(pass.is_empty());
        assert_eq!(completed.len(), 2);
        assert_eq!(completed[0], b"a=t;img1");
        assert_eq!(completed[1], b"a=p;img2");
    }

    #[test]
    fn test_esc_at_boundary() {
        let mut filter = ApcFilter::new();

        // ESC at end of first chunk
        let (pass1, completed1) = filter.filter(b"data\x1b");
        assert_eq!(pass1, b"data");
        assert!(completed1.is_empty());

        // _ G at start of second chunk
        let (pass2, completed2) = filter.filter(b"_Ga=t;x\x1b\\");
        assert!(pass2.is_empty());
        assert_eq!(completed2.len(), 1);
        assert_eq!(completed2[0], b"a=t;x");
    }
}
