/// Utilities to retrieve input state.
#mod state
#import ~.~.math.util

/// The state of an input (e.g. a button).
alias InputState = u32;

/// Returns whether an input is pressed.
fn is_pressed(state: InputState) -> bool {
    const BIT = 0;
    return (state & (1 << BIT)) != 0;
}

/// Returns whether an input has just been pressed.
fn is_just_pressed(state: InputState) -> bool {
    const BIT = 1;
    return (state & (1 << BIT)) != 0;
}

/// Returns whether an input has just been released.
fn is_just_released(state: InputState) -> bool {
    const BIT = 2;
    return (state & (1 << BIT)) != 0;
}

/// Returns the input ID when specified.
///
/// This is only useful for special mouse buttons.
fn input_id(state: InputState) -> u32 {
    const BIT_OFFSET = 16;
    return state >> BIT_OFFSET;
}

/// Returns the normalized direction based on left, right, up and down inputs.
///
/// If none of the inputs are pressed, the returned direction is `vec2f(0, 0)`.
fn input_direction(left: InputState, right: InputState, up: InputState, down: InputState) -> vec2f {
    return normalize_vec2f_or_zero(vec2f(input_axis(left, right), input_axis(down, up)));
}

/// Returns the axis value between between -1. and 1 based on left and right inputs.
///
/// If none or both inputs are pressed, the returned axis is 0.
fn input_axis(left: InputState, right: InputState) -> f32 {
    return select(0., -1., is_pressed(left)) + select(0., 1., is_pressed(right));
}


/// Keyboard constants.
#mod keyboard

/// <kbd>`</kbd> on a US keyboard. This is also called a backtick or grave.
/// This is the <kbd>半角</kbd>/<kbd>全角</kbd>/<kbd>漢字</kbd>
/// (hankaku/zenkaku/kanji) key on Japanese keyboards
const KB_BACKQUOTE = 0;
/// Used for both the US <kbd>\\</kbd> (on the 101-key layout) and also for the key
/// located between the <kbd>"</kbd> and <kbd>Enter</kbd> keys on row C of the 102-,
/// 104- and 106-key layouts.
/// Labeled <kbd>#</kbd> on a UK (102) keyboard.
const KB_BACKSLASH = 1;
/// <kbd>[</kbd> on a US keyboard.
const KB_BRACKET_LEFT = 2;
/// <kbd>]</kbd> on a US keyboard.
const KB_BRACKET_RIGHT = 3;
/// <kbd>,</kbd> on a US keyboard.
const KB_COMMA = 4;
/// <kbd>0</kbd> on a US keyboard.
const KB_DIGIT_0 = 5;
/// <kbd>1</kbd> on a US keyboard.
const KB_DIGIT_1 = 6;
/// <kbd>2</kbd> on a US keyboard.
const KB_DIGIT_2 = 7;
/// <kbd>3</kbd> on a US keyboard.
const KB_DIGIT_3 = 8;
/// <kbd>4</kbd> on a US keyboard.
const KB_DIGIT_4 = 9;
/// <kbd>5</kbd> on a US keyboard.
const KB_DIGIT_5 = 10;
/// <kbd>6</kbd> on a US keyboard.
const KB_DIGIT_6 = 11;
/// <kbd>7</kbd> on a US keyboard.
const KB_DIGIT_7 = 12;
/// <kbd>8</kbd> on a US keyboard.
const KB_DIGIT_8 = 13;
/// <kbd>9</kbd> on a US keyboard.
const KB_DIGIT_9 = 14;
/// <kbd>=</kbd> on a US keyboard.
const KB_EQUAL = 15;
/// Located between the left <kbd>Shift</kbd> and <kbd>Z</kbd> keys.
/// Labeled <kbd>\\</kbd> on a UK keyboard.
const KB_INTL_BACKSLASH = 16;
/// Located between the <kbd>/</kbd> and right <kbd>Shift</kbd> keys.
/// Labeled <kbd>\\</kbd> (ro) on a Japanese keyboard.
const KB_INTL_RO = 17;
/// Located between the <kbd>=</kbd> and <kbd>Backspace</kbd> keys.
/// Labeled <kbd>¥</kbd> (yen) on a Japanese keyboard. <kbd>\\</kbd> on a
/// Russian keyboard.
const KB_INTL_YEN = 18;
/// <kbd>a</kbd> on a US keyboard.
/// Labeled <kbd>q</kbd> on an AZERTY (e.g., French) keyboard.
const KB_KEY_A = 19;
/// <kbd>b</kbd> on a US keyboard.
const KB_KEY_B = 20;
/// <kbd>c</kbd> on a US keyboard.
const KB_KEY_C = 21;
/// <kbd>d</kbd> on a US keyboard.
const KB_KEY_D = 22;
/// <kbd>e</kbd> on a US keyboard.
const KB_KEY_E = 23;
/// <kbd>f</kbd> on a US keyboard.
const KB_KEY_F = 24;
/// <kbd>g</kbd> on a US keyboard.
const KB_KEY_G = 25;
/// <kbd>h</kbd> on a US keyboard.
const KB_KEY_H = 26;
/// <kbd>i</kbd> on a US keyboard.
const KB_KEY_I = 27;
/// <kbd>j</kbd> on a US keyboard.
const KB_KEY_J = 28;
/// <kbd>k</kbd> on a US keyboard.
const KB_KEY_K = 29;
/// <kbd>l</kbd> on a US keyboard.
const KB_KEY_L = 30;
/// <kbd>m</kbd> on a US keyboard.
const KB_KEY_M = 31;
/// <kbd>n</kbd> on a US keyboard.
const KB_KEY_N = 32;
/// <kbd>o</kbd> on a US keyboard.
const KB_KEY_O = 33;
/// <kbd>p</kbd> on a US keyboard.
const KB_KEY_P = 34;
/// <kbd>q</kbd> on a US keyboard.
/// Labeled <kbd>a</kbd> on an AZERTY (e.g., French) keyboard.
const KB_KEY_Q = 35;
/// <kbd>r</kbd> on a US keyboard.
const KB_KEY_R = 36;
/// <kbd>s</kbd> on a US keyboard.
const KB_KEY_S = 37;
/// <kbd>t</kbd> on a US keyboard.
const KB_KEY_T = 38;
/// <kbd>u</kbd> on a US keyboard.
const KB_KEY_U = 39;
/// <kbd>v</kbd> on a US keyboard.
const KB_KEY_V = 40;
/// <kbd>w</kbd> on a US keyboard.
/// Labeled <kbd>z</kbd> on an AZERTY (e.g., French) keyboard.
const KB_KEY_W = 41;
/// <kbd>x</kbd> on a US keyboard.
const KB_KEY_X = 42;
/// <kbd>y</kbd> on a US keyboard.
/// Labeled <kbd>z</kbd> on a QWERTZ (e.g., German) keyboard.
const KB_KEY_Y = 43;
/// <kbd>z</kbd> on a US keyboard.
/// Labeled <kbd>w</kbd> on an AZERTY (e.g., French) keyboard, and <kbd>y</kbd> on a
/// QWERTZ (e.g., German) keyboard.
const KB_KEY_Z = 44;
/// <kbd>-</kbd> on a US keyboard.
const KB_MINUS = 45;
/// <kbd>.</kbd> on a US keyboard.
const KB_PERIOD = 46;
/// <kbd>'</kbd> on a US keyboard.
const KB_QUOTE = 47;
/// <kbd>;</kbd> on a US keyboard.
const KB_SEMICOLON = 48;
/// <kbd>/</kbd> on a US keyboard.
const KB_SLASH = 49;
/// <kbd>Alt</kbd>, <kbd>Option</kbd>, or <kbd>⌥</kbd>.
const KB_ALT_LEFT = 50;
/// <kbd>Alt</kbd>, <kbd>Option</kbd>, or <kbd>⌥</kbd>.
/// This is labeled <kbd>AltGr</kbd> on many keyboard layouts.
const KB_ALT_RIGHT = 51;
/// <kbd>Backspace</kbd> or <kbd>⌫</kbd>.
/// Labeled <kbd>Delete</kbd> on Apple keyboards.
const KB_BACKSPACE = 52;
/// <kbd>CapsLock</kbd> or <kbd>⇪</kbd>
const KB_CAPS_LOCK = 53;
/// The application context menu key, which is typically found between the right
/// <kbd>Super</kbd> key and the right <kbd>Control</kbd> key.
const KB_CONTEXT_MENU = 54;
/// <kbd>Control</kbd> or <kbd>⌃</kbd>
const KB_CONTROL_LEFT = 55;
/// <kbd>Control</kbd> or <kbd>⌃</kbd>
const KB_CONTROL_RIGHT = 56;
/// <kbd>Enter</kbd> or <kbd>↵</kbd>. Labeled <kbd>Return</kbd> on Apple keyboards.
const KB_ENTER = 57;
/// The Windows, <kbd>⌘</kbd>, <kbd>Command</kbd>, or other OS symbol key.
const KB_SUPER_LEFT = 58;
/// The Windows, <kbd>⌘</kbd>, <kbd>Command</kbd>, or other OS symbol key.
const KB_SUPER_RIGHT = 59;
/// <kbd>Shift</kbd> or <kbd>⇧</kbd>
const KB_SHIFT_LEFT = 60;
/// <kbd>Shift</kbd> or <kbd>⇧</kbd>
const KB_SHIFT_RIGHT = 61;
/// <kbd> </kbd> (space)
const KB_SPACE = 62;
/// <kbd>Tab</kbd> or <kbd>⇥</kbd>
const KB_TAB = 63;
/// Japanese: <kbd>変</kbd> (henkan)
const KB_CONVERT = 64;
/// Japanese: <kbd>カタカナ</kbd>/<kbd>ひらがな</kbd>/<kbd>ローマ字</kbd> (katakana/hiragana/romaji)
const KB_KANA_MODE = 65;
/// Korean: HangulMode <kbd>한/영</kbd> (han/yeong)
///
/// Japanese (Mac keyboard): <kbd>か</kbd> (kana)
const KB_LANG1 = 66;
/// Korean: Hanja <kbd>한</kbd> (hanja)
///
/// Japanese (Mac keyboard): <kbd>英</kbd> (eisu)
const KB_LANG2 = 67;
/// Japanese (word-processing keyboard): Katakana
const KB_LANG3 = 68;
/// Japanese (word-processing keyboard): Hiragana
const KB_LANG4 = 69;
/// Japanese (word-processing keyboard): Zenkaku/Hankaku
const KB_LANG5 = 70;
/// Japanese: <kbd>無変換</kbd> (muhenkan)
const KB_NON_CONVERT = 71;
/// <kbd>⌦</kbd>. The forward delete key.
/// Note that on Apple keyboards, the key labelled <kbd>Delete</kbd> on the main part of
/// the keyboard is encoded as [`Key::Backspace`].
const KB_DELETE = 72;
/// <kbd>Page Down</kbd>, <kbd>End</kbd>, or <kbd>↘</kbd>
const KB_END = 73;
/// <kbd>Help</kbd>. Not present on standard PC keyboards.
const KB_HELP = 74;
/// <kbd>Home</kbd> or <kbd>↖</kbd>
const KB_HOME = 75;
/// <kbd>Insert</kbd> or <kbd>Ins</kbd>. Not present on Apple keyboards.
const KB_INSERT = 76;
/// <kbd>Page Down</kbd>, <kbd>PgDn</kbd>, or <kbd>⇟</kbd>
const KB_PAGE_DOWN = 77;
/// <kbd>Page Up</kbd>, <kbd>PgUp</kbd>, or <kbd>⇞</kbd>
const KB_PAGE_UP = 78;
/// <kbd>↓</kbd>
const KB_ARROW_DOWN = 79;
/// <kbd>←</kbd>
const KB_ARROW_LEFT = 80;
/// <kbd>→</kbd>
const KB_ARROW_RIGHT = 81;
/// <kbd>↑</kbd>
const KB_ARROW_UP = 82;
/// On the Mac, this is used for the numpad <kbd>Clear</kbd> key.
const KB_NUM_LOCK = 83;
/// <kbd>0 Ins</kbd> on a keyboard. <kbd>0</kbd> on a phone or remote control
const KB_NUMPAD_0 = 84;
/// <kbd>1 End</kbd> on a keyboard. <kbd>1</kbd> or <kbd>1 QZ</kbd> on a phone or remote control
const KB_NUMPAD_1 = 85;
/// <kbd>2 ↓</kbd> on a keyboard. <kbd>2 ABC</kbd> on a phone or remote control
const KB_NUMPAD_2 = 86;
/// <kbd>3 PgDn</kbd> on a keyboard. <kbd>3 DEF</kbd> on a phone or remote control
const KB_NUMPAD_3 = 87;
/// <kbd>4 ←</kbd> on a keyboard. <kbd>4 GHI</kbd> on a phone or remote control
const KB_NUMPAD_4 = 88;
/// <kbd>5</kbd> on a keyboard. <kbd>5 JKL</kbd> on a phone or remote control
const KB_NUMPAD_5 = 89;
/// <kbd>6 →</kbd> on a keyboard. <kbd>6 MNO</kbd> on a phone or remote control
const KB_NUMPAD_6 = 90;
/// <kbd>7 Home</kbd> on a keyboard. <kbd>7 PQRS</kbd> or <kbd>7 PRS</kbd> on a phone
/// or remote control
const KB_NUMPAD_7 = 91;
/// <kbd>8 ↑</kbd> on a keyboard. <kbd>8 TUV</kbd> on a phone or remote control
const KB_NUMPAD_8 = 92;
/// <kbd>9 PgUp</kbd> on a keyboard. <kbd>9 WXYZ</kbd> or <kbd>9 WXY</kbd> on a phone
/// or remote control
const KB_NUMPAD_9 = 93;
/// <kbd>+</kbd>
const KB_NUMPAD_ADD = 94;
/// Found on the Microsoft Natural Keyboard.
const KB_NUMPAD_BACKSPACE = 95;
/// <kbd>C</kbd> or <kbd>A</kbd> (All Clear). Also for use with numpads that have a
/// <kbd>Clear</kbd> key that is separate from the <kbd>NumLock</kbd> key. On the Mac, the
/// numpad <kbd>Clear</kbd> key is encoded as [`Key::NumLock`].
const KB_NUMPAD_CLEAR = 96;
/// <kbd>C</kbd> (Clear Entry)
const KB_NUMPAD_CLEAR_ENTRY = 97;
/// <kbd>,</kbd> (thousands separator). For locales where the thousands separator
/// is a "." (e.g., Brazil), this key may generate a <kbd>.</kbd>.
const KB_NUMPAD_COMMA = 98;
/// <kbd>. Del</kbd>. For locales where the decimal separator is "," (e.g.,
/// Brazil), this key may generate a <kbd>,</kbd>.
const KB_NUMPAD_DECIMAL = 99;
/// <kbd>/</kbd>
const KB_NUMPAD_DIVIDE = 100;
/// <kbd>Enter</kbd> or <kbd>↵</kbd> in numpad.
const KB_NUMPAD_ENTER = 101;
/// <kbd>=</kbd>
const KB_NUMPAD_EQUAL = 102;
/// <kbd>#</kbd> on a phone or remote control device. This key is typically found
/// below the <kbd>9</kbd> key and to the right of the <kbd>0</kbd> key.
const KB_NUMPAD_HASH = 103;
/// <kbd>M</kbd> Add current entry to the value stored in memory.
const KB_NUMPAD_MEMORY_ADD = 104;
/// <kbd>M</kbd> Clear the value stored in memory.
const KB_NUMPAD_MEMORY_CLEAR = 105;
/// <kbd>M</kbd> Replace the current entry with the value stored in memory.
const KB_NUMPAD_MEMORY_RECALL = 106;
/// <kbd>M</kbd> Replace the value stored in memory with the current entry.
const KB_NUMPAD_MEMORY_STORE = 107;
/// <kbd>M</kbd> Subtract current entry from the value stored in memory.
const KB_NUMPAD_MEMORY_SUBTRACT = 108;
/// <kbd>*</kbd> on a keyboard. For use with numpads that provide mathematical
/// operations (<kbd>+</kbd>, <kbd>-</kbd> <kbd>*</kbd> and <kbd>/</kbd>).
///
/// Use [`Key::NumpadStar`] for the <kbd>*</kbd> key on phones and remote controls.
const KB_NUMPAD_MULTIPLY = 109;
/// <kbd>(</kbd> Found on the Microsoft Natural Keyboard.
const KB_NUMPAD_PAREN_LEFT = 110;
/// <kbd>)</kbd> Found on the Microsoft Natural Keyboard.
const KB_NUMPAD_PAREN_RIGHT = 111;
/// <kbd>*</kbd> on a phone or remote control device.
///
/// This key is typically found below the <kbd>7</kbd> key and to the left of
/// the <kbd>0</kbd> key.
///
/// Use <kbd>"NumpadMultiply"</kbd> for the <kbd>*</kbd> key on
/// numeric keypads.
const KB_NUMPAD_STAR = 112;
/// <kbd>-</kbd>
const KB_NUMPAD_SUBTRACT = 113;
/// <kbd>Esc</kbd> or <kbd>⎋</kbd>
const KB_ESCAPE = 114;
/// <kbd>Fn</kbd> This is typically a hardware key that does not generate a separate code.
const KB_FN = 115;
/// <kbd>FLock</kbd> or <kbd>FnLock</kbd>. Function Lock key. Found on the Microsoft
/// Natural Keyboard.
const KB_FN_LOCK = 116;
/// <kbd>PrtScr SysRq</kbd> or <kbd>Print Screen</kbd>
const KB_PRINT_SCREEN = 117;
/// <kbd>Scroll Lock</kbd>
const KB_SCROLL_LOCK = 118;
/// <kbd>Pause Break</kbd>
const KB_PAUSE = 119;
/// Some laptops place this key to the left of the <kbd>↑</kbd> key.
///
/// This also the "back" button (triangle) on Android.
const KB_BROWSER_BACK = 120;
const KB_BROWSER_FAVORITES = 121;
/// Some laptops place this key to the right of the <kbd>↑</kbd> key.
const KB_BROWSER_FORWARD = 122;
/// The "home" button on Android.
const KB_BROWSER_HOME = 123;
const KB_BROWSER_REFRESH = 124;
const KB_BROWSER_SEARCH = 125;
const KB_BROWSER_STOP = 126;
/// <kbd>Eject</kbd> or <kbd>⏏</kbd>. This key is placed in the function section on some Apple
/// keyboards.
const KB_EJECT = 127;
/// Sometimes labeled <kbd>My Computer</kbd> on the keyboard
const KB_LAUNCH_APP1 = 128;
/// Sometimes labeled <kbd>Calculator</kbd> on the keyboard
const KB_LAUNCH_APP2 = 129;
const KB_LAUNCH_MAIL = 130;
const KB_MEDIA_PLAY_PAUSE = 131;
const KB_MEDIA_SELECT = 132;
const KB_MEDIA_STOP = 133;
const KB_MEDIA_TRACK_NEXT = 134;
const KB_MEDIA_TRACK_PREVIOUS = 135;
/// This key is placed in the function section on some Apple keyboards, replacing the
/// <kbd>Eject</kbd> key.
const KB_POWER = 136;
const KB_SLEEP = 137;
const KB_AUDIO_VOLUME_DOWN = 138;
const KB_AUDIO_VOLUME_MUTE = 139;
const KB_AUDIO_VOLUME_UP = 140;
/// Legacy modifier key.
const KB_WAKE_UP = 141;
const KB_META = 142;
/// Legacy modifier key.
const KB_HYPER = 143;
const KB_TURBO = 144;
const KB_ABORT = 145;
const KB_RESUME = 146;
const KB_SUSPEND = 147;
/// Found on Sun’s USB keyboard.
const KB_AGAIN = 148;
/// Found on Sun’s USB keyboard.
const KB_COPY = 149;
/// Found on Sun’s USB keyboard.
const KB_CUT = 150;
/// Found on Sun’s USB keyboard.
const KB_FIND = 151;
/// Found on Sun’s USB keyboard.
const KB_OPEN = 152;
/// Found on Sun’s USB keyboard.
const KB_PASTE = 153;
/// Found on Sun’s USB keyboard.
const KB_PROPS = 154;
/// Found on Sun’s USB keyboard.
const KB_SELECT = 155;
/// Found on Sun’s USB keyboard.
const KB_UNDO = 156;
/// Use for dedicated <kbd>ひらがな</kbd> key found on some Japanese word processing keyboards.
const KB_HIRAGANA = 157;
/// Use for dedicated <kbd>カタカナ</kbd> key found on some Japanese word processing keyboards.
const KB_KATAKANA = 158;
/// General-purpose function key.
/// Usually found at the top of the keyboard.
const KB_F1 = 159;
/// General-purpose function key.
/// Usually found at the top of the keyboard.
const KB_F2 = 160;
/// General-purpose function key.
/// Usually found at the top of the keyboard.
const KB_F3 = 161;
/// General-purpose function key.
/// Usually found at the top of the keyboard.
const KB_F4 = 162;
/// General-purpose function key.
/// Usually found at the top of the keyboard.
const KB_F5 = 163;
/// General-purpose function key.
/// Usually found at the top of the keyboard.
const KB_F6 = 164;
/// General-purpose function key.
/// Usually found at the top of the keyboard.
const KB_F7 = 165;
/// General-purpose function key.
/// Usually found at the top of the keyboard.
const KB_F8 = 166;
/// General-purpose function key.
/// Usually found at the top of the keyboard.
const KB_F9 = 167;
/// General-purpose function key.
/// Usually found at the top of the keyboard.
const KB_F10 = 168;
/// General-purpose function key.
/// Usually found at the top of the keyboard.
const KB_F11 = 169;
/// General-purpose function key.
/// Usually found at the top of the keyboard.
const KB_F12 = 170;
/// General-purpose function key.
/// Usually found at the top of the keyboard.
const KB_F13 = 171;
/// General-purpose function key.
/// Usually found at the top of the keyboard.
const KB_F14 = 172;
/// General-purpose function key.
/// Usually found at the top of the keyboard.
const KB_F15 = 173;
/// General-purpose function key.
/// Usually found at the top of the keyboard.
const KB_F16 = 174;
/// General-purpose function key.
/// Usually found at the top of the keyboard.
const KB_F17 = 175;
/// General-purpose function key.
/// Usually found at the top of the keyboard.
const KB_F18 = 176;
/// General-purpose function key.
/// Usually found at the top of the keyboard.
const KB_F19 = 177;
/// General-purpose function key.
/// Usually found at the top of the keyboard.
const KB_F20 = 178;
/// General-purpose function key.
/// Usually found at the top of the keyboard.
const KB_F21 = 179;
/// General-purpose function key.
/// Usually found at the top of the keyboard.
const KB_F22 = 180;
/// General-purpose function key.
/// Usually found at the top of the keyboard.
const KB_F23 = 181;
/// General-purpose function key.
/// Usually found at the top of the keyboard.
const KB_F24 = 182;
/// General-purpose function key.
const KB_F25 = 183;
/// General-purpose function key.
const KB_F26 = 184;
/// General-purpose function key.
const KB_F27 = 185;
/// General-purpose function key.
const KB_F28 = 186;
/// General-purpose function key.
const KB_F29 = 187;
/// General-purpose function key.
const KB_F30 = 188;
/// General-purpose function key.
const KB_F31 = 189;
/// General-purpose function key.
const KB_F32 = 190;
/// General-purpose function key.
const KB_F33 = 191;
/// General-purpose function key.
const KB_F34 = 192;
/// General-purpose function key.
const KB_F35 = 193;


/// Keyboard constants.
#mod mouse

/// Left mouse button.
const MS_BUTTON_LEFT = 0;
/// Right mouse button.
const MS_BUTTON_RIGHT = 1;
/// Middle/wheel mouse button.
const MS_BUTTON_MIDDLE = 2;
/// Back mouse button.
const MS_BUTTON_BACK = 3;
/// Forward mouse button.
const MS_BUTTON_FORWARD = 4;

/// Mouse wheel unit in lines (row and columns).
const MS_WHEEL_LINES = 0;
/// Mouse wheel unit in pixels.
const MS_WHEEL_PIXELS = 1;
